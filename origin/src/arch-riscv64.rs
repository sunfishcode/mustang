use linux_raw_sys::general::{__NR_clone, __NR_exit, __NR_munmap};
use rsix::process::RawPid;
use std::ffi::c_void;

#[inline]
pub(super) unsafe fn clone(
    flags: u32,
    child_stack: *mut c_void,
    ptid: *mut RawPid,
    newtls: *mut c_void,
    ctid: *mut RawPid,
    arg: *mut c_void,
    fn_: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
) -> isize {
    let r0;
    asm!(
        "ecall",
        "bnez a0,0f",

        // Child thread.
        "mov a0,{arg}",       // `arg`
        "mov a1,{fn_}",       // `fn_`
        "mov fp, xzr",        // zero the frame address
        "mov ra, xzr",        // zero the return address
        "b {entry}",

        // Parent thread.
        "0:",

        entry = sym super::threads::entry,
        arg = in(reg) arg,
        fn_ = in(reg) fn_,
        in("a7") __NR_clone,
        inlateout("a0") flags as isize => r0,
        in("a1") child_stack,
        in("a2") ptid,
        in("a3") newtls,
        in("a4") ctid,
        options(nostack)
    );
    r0
}

#[inline]
pub(super) unsafe fn set_thread_pointer(thread_data: *mut c_void) {
    asm!("mov tp,{0}", in(reg) thread_data);
    debug_assert_eq!(thread_self(), thread_data);
}

#[inline]
pub(super) fn thread_self() -> *mut c_void {
    let result;
    unsafe {
        asm!("mv {0},tp", out(reg) result, options(nostack, preserves_flags, readonly));
    }
    result
}

/// `munmap` the thread, then carefully exit the thread without touching the
/// deallocated stack.
#[inline]
pub(super) unsafe fn deallocate_thread(addr: *mut c_void, len: usize) -> ! {
    asm!(
        "ecall",
        "mov a0,xzr",
        "mov a7,{__NR_exit}",
        "ecall",
        "unimp",
        __NR_exit = const __NR_exit,
        in("a7") __NR_munmap,
        in("a0") addr,
        in("a1") len,
        options(noreturn, nostack, preserves_flags)
    );
}
