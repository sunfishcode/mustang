use linux_raw_sys::general::{__NR_clone, __NR_exit, __NR_munmap};
use rsix::process::RawPid;
use std::ffi::c_void;

#[inline]
pub(super) unsafe fn clone(
    flags: u32,
    child_stack: *mut c_void,
    parent_tid: *mut RawPid,
    newtls: *mut c_void,
    child_tid: *mut RawPid,
    arg: *mut c_void,
    fn_: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
) -> isize {
    let r0;
    asm!(
        "svc 0",
        "cbnz x0,0f",

        // Child thread.
        "mov x0,{arg}",       // `arg`
        "mov x1,{fn_}",       // `fn_`
        "mov x29, xzr",       // zero the frame address
        "mov x30, xzr",       // zero the return address
        "b {entry}",

        // Parent thread.
        "0:",

        entry = sym super::threads::entry,
        arg = in(reg) arg,
        fn_ = in(reg) fn_,
        in("x8") __NR_clone,
        inlateout("x0") flags as isize => r0,
        in("x1") child_stack,
        in("x2") parent_tid,
        in("x3") newtls,
        in("x4") child_tid,
        options(nostack)
    );
    r0
}

#[inline]
pub(super) unsafe fn set_thread_pointer(ptr: *mut c_void) {
    asm!("msr tpidr_el0,{0}", in(reg) ptr);
    debug_assert_eq!(get_thread_pointer(), ptr);
}

#[inline]
pub(super) fn get_thread_pointer() -> *mut c_void {
    let ptr;
    unsafe {
        asm!("mrs {0},tpidr_el0", out(reg) ptr, options(nostack, preserves_flags, readonly));
    }
    ptr
}

/// `munmap` the current thread, then carefully exit the thread without
/// touching the deallocated stack.
#[inline]
pub(super) unsafe fn munmap_and_exit_thread(map_addr: *mut c_void, map_len: usize) -> ! {
    asm!(
        "svc 0",
        "mov x0,xzr",
        "mov x8,{__NR_exit}",
        "svc 0",
        "udf #16",
        __NR_exit = const __NR_exit,
        in("x8") __NR_munmap,
        in("x0") map_addr,
        in("x1") map_len,
        options(noreturn, nostack)
    );
}
