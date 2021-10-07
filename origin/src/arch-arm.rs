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
        "tst r0,r0",
        "bne 0f",

        // Child thread.
        "mov r0,{arg}",       // `arg`
        "mov r1,{fn_}",       // `fn_`
        "mov fp, #0",         // zero the frame address
        "mov lr, #0",         // zero the return address
        "b {entry}",

        // Parent thread.
        "0:",

        entry = sym super::threads::entry,
        arg = in(reg) arg,
        fn_ = in(reg) fn_,
        in("r7") __NR_clone,
        inlateout("r0") flags as isize => r0,
        in("r1") child_stack,
        in("r2") parent_tid,
        in("r3") newtls,
        in("r4") child_tid,
        options(nostack)
    );
    r0
}

#[cfg(target_vendor = "mustang")]
#[inline]
pub(super) unsafe fn set_thread_pointer(ptr: *mut c_void) {
    rsix::runtime::arm_set_tls(ptr).expect("arm_set_tls");
    debug_assert_eq!(get_thread_pointer(), ptr);
}

#[inline]
pub(super) fn get_thread_pointer() -> *mut c_void {
    let ptr;
    unsafe {
        asm!("mrc p15,0,{0},c13,c0,3", out(reg) ptr);
    }
    ptr
}

/// TLS data starts at the location pointed to by the thread pointer.
pub(super) const TLS_OFFSET: usize = 0;

/// `munmap` the current thread, then carefully exit the thread without
/// touching the deallocated stack.
#[inline]
pub(super) unsafe fn munmap_and_exit_thread(map_addr: *mut c_void, map_len: usize) -> ! {
    asm!(
        "svc 0",
        "mov r0,#0",
        "mov r7,{__NR_exit}",
        "svc 0",
        "udf #16",
        __NR_exit = const __NR_exit,
        in("r7") __NR_munmap,
        in("r0") map_addr,
        in("r1") map_len,
        options(noreturn, nostack)
    );
}
