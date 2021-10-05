use linux_raw_sys::general::{__NR_clone, __NR_exit, __NR_munmap};
use rsix::process::RawPid;
use std::ffi::c_void;

#[inline]
pub(super) unsafe fn clone(
    flags: u32,
    child_stack: *mut c_void,
    parent_tid: *mut RawPid,
    child_tid: *mut RawPid,
    newtls: *mut c_void,
    arg: *mut c_void,
    fn_: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
) -> isize {
    let r0;
    asm!(
        // Pass `fn_` and `arg` to the child thread. We don't have enough
        // registers, so `arg` has to go on the child thread's stack.
        "mov    rcx,QWORD PTR [r9]",
        "mov    QWORD PTR [rsi-8],rcx",
        "mov    r9,QWORD PTR [r9+8]",

        "syscall",
        "test   eax,eax",
        "jnz    0f",

        // Child thread.
        "xor    ebp,ebp",     // zero the frame pointer
        "mov    rdi,[rsp-8]", // `arg`
        "mov    rsi,r9",      // `fn_`
        "push   rbp",         // zero the return address
        "jmp    {entry}",

        // Parent thread.
        "0:",

        entry = sym super::threads::entry,
        inlateout("rax") __NR_clone as isize => r0,
        in("rdi") flags,
        in("rsi") child_stack,
        in("rdx") parent_tid,
        in("r10") child_tid,
        in("r8") newtls,
        in("r9") &[arg, fn_ as _],
        lateout("rcx") _,
        lateout("r11") _,
        options(nostack)
    );
    r0
}

#[cfg(target_vendor = "mustang")]
#[inline]
pub(super) unsafe fn set_thread_pointer(ptr: *mut c_void) {
    rsix::runtime::set_fs(ptr);
    debug_assert_eq!(*ptr.cast::<*const c_void>(), ptr);
    debug_assert_eq!(get_thread_pointer(), ptr);
}

#[inline]
pub(super) fn get_thread_pointer() -> *mut c_void {
    let ptr;
    unsafe {
        asm!("mov {},QWORD PTR fs:0", out(reg) ptr, options(nostack, preserves_flags, readonly));
    }
    ptr
}

/// `munmap` the current thread, then carefully exit the thread without
/// touching the deallocated stack.
#[inline]
pub(super) unsafe fn munmap_and_exit_thread(map_addr: *mut c_void, map_len: usize) -> ! {
    asm!(
        "syscall",
        "xor edi,edi",
        "mov eax,{__NR_exit}",
        "syscall",
        "ud2",
        __NR_exit = const __NR_exit,
        in("rax") __NR_munmap,
        in("rdi") map_addr,
        in("rsi") map_len,
        options(noreturn, nostack)
    );
}
