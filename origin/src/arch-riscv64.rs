use alloc::boxed::Box;
use core::any::Any;
use core::arch::asm;
use core::ffi::c_void;
use linux_raw_sys::general::{__NR_clone, __NR_exit, __NR_munmap};
use rustix::process::RawPid;

/// A wrapper around the Linux `clone` system call.
#[inline]
pub(super) unsafe fn clone(
    flags: u32,
    child_stack: *mut c_void,
    parent_tid: *mut RawPid,
    newtls: *mut c_void,
    child_tid: *mut RawPid,
    fn_: *mut Box<dyn FnOnce() -> Option<Box<dyn Any>>>,
) -> isize {
    let r0;
    asm!(
        "ecall",
        "bnez a0,0f",

        // Child thread.
        "mv a0,{fn_}",        // `fn_`
        "mv fp, zero",        // zero the frame address
        "mv ra, zero",        // zero the return address
        "tail {entry}",

        // Parent thread.
        "0:",

        entry = sym super::threads::entry,
        fn_ = in(reg) fn_,
        in("a7") __NR_clone,
        inlateout("a0") flags as isize => r0,
        in("a1") child_stack,
        in("a2") parent_tid,
        in("a3") newtls,
        in("a4") child_tid,
        options(nostack)
    );
    r0
}

/// Write a value to the platform thread-pointer register.
#[cfg(target_vendor = "mustang")]
#[inline]
pub(super) unsafe fn set_thread_pointer(ptr: *mut c_void) {
    asm!("mv tp,{0}", in(reg) ptr);
    debug_assert_eq!(get_thread_pointer(), ptr);
}

/// Read the value of the platform thread-pointer register.
#[inline]
pub(super) fn get_thread_pointer() -> *mut c_void {
    let ptr;
    unsafe {
        asm!("mv {0},tp", out(reg) ptr, options(nostack, preserves_flags, readonly));
    }
    ptr
}

/// TLS data starts 0x800 bytes below the location pointed to by the thread
/// pointer.
pub(super) const TLS_OFFSET: usize = 0x800;

/// `munmap` the current thread, then carefully exit the thread without
/// touching the deallocated stack.
#[inline]
pub(super) unsafe fn munmap_and_exit_thread(map_addr: *mut c_void, map_len: usize) -> ! {
    asm!(
        "ecall",
        "mv a0,zero",
        "li a7,{__NR_exit}",
        "ecall",
        "unimp",
        __NR_exit = const __NR_exit,
        in("a7") __NR_munmap,
        in("a0") map_addr,
        in("a1") map_len,
        options(noreturn, nostack)
    );
}
