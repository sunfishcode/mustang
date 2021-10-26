use linux_raw_sys::general::{__NR_clone, __NR_exit, __NR_munmap};
use rsix::process::RawPid;
use std::any::Any;
use std::ffi::c_void;

#[inline]
pub(super) unsafe fn clone(
    flags: u32,
    child_stack: *mut c_void,
    parent_tid: *mut RawPid,
    newtls: *mut c_void,
    child_tid: *mut RawPid,
    fn_: *mut Box<dyn FnOnce() -> Option<Box<dyn Any>>>,
) -> isize {
    let mut gs: u32 = 0;
    asm!("mov {0:x},gs", inout(reg) gs);
    let entry_number = gs >> 3;

    let mut user_desc = rsix::runtime::UserDesc {
        entry_number,
        base_addr: newtls as u32,
        limit: 0xfffff,
        _bitfield_align_1: [],
        _bitfield_1: Default::default(),
        __bindgen_padding_0: [0_u8; 3_usize],
    };
    user_desc.set_seg_32bit(1);
    user_desc.set_contents(0);
    user_desc.set_read_exec_only(0);
    user_desc.set_limit_in_pages(1);
    user_desc.set_seg_not_present(0);
    user_desc.set_useable(1);
    let newtls: *const _ = &user_desc;

    // See the comments for x86's `syscall6` in rsix. Inline asm isn't
    // allowed to name ebp or esi as operands, so we have to jump through
    // extra hoops here.
    let r0;
    asm!(
        "push esi",
        "push ebp",

        // Pass `fn_` to the child in ebp.
        "mov ebp,DWORD PTR [eax+8]",

        "mov esi, [eax + 0]", // `newtls`
        "mov eax, [eax + 4]", // `__NR_clone`

        // Use `int 0x80` instead of vsyscall, following `clone`'s
        // documentation; vsyscall would attempt to return to the parent stack
        // in the child.
        "int 0x80",
        "test eax,eax",
        "jnz 0f",

        // Child thread.
        "push eax",           // pad for alignment
        "push eax",           // pad for alignment
        "push eax",           // pad for alignment
        "push ebp",           // `fn`
        "xor ebp,ebp",        // zero the frame address
        "push eax",           // zero the return address
        "jmp {entry}",

        // Parent thread.
        "0:",
        "pop ebp",
        "pop esi",

        entry = sym super::threads::entry,
        inout("eax") &[newtls as usize, __NR_clone as usize, fn_ as usize] => r0,
        in("ebx") flags,
        in("ecx") child_stack,
        in("edx") parent_tid,
        in("edi") child_tid,
    );
    r0
}

#[cfg(target_vendor = "mustang")]
#[inline]
pub(super) unsafe fn set_thread_pointer(ptr: *mut c_void) {
    let mut user_desc = rsix::runtime::UserDesc {
        entry_number: -1i32 as u32,
        base_addr: ptr as u32,
        limit: 0xfffff,
        _bitfield_align_1: [],
        _bitfield_1: Default::default(),
        __bindgen_padding_0: [0_u8; 3_usize],
    };
    user_desc.set_seg_32bit(1);
    user_desc.set_contents(0);
    user_desc.set_read_exec_only(0);
    user_desc.set_limit_in_pages(1);
    user_desc.set_seg_not_present(0);
    user_desc.set_useable(1);
    rsix::runtime::set_thread_area(&mut user_desc).expect("set_thread_area");
    asm!("mov gs,{0:x}", in(reg) ((user_desc.entry_number << 3) | 3) as u16);
    debug_assert_eq!(*ptr.cast::<*const c_void>(), ptr);
    debug_assert_eq!(get_thread_pointer(), ptr);
}

#[inline]
pub(super) fn get_thread_pointer() -> *mut c_void {
    let ptr;
    unsafe {
        asm!("mov {0},DWORD PTR gs:0", out(reg) ptr, options(nostack, preserves_flags, readonly));
    }
    ptr
}

/// TLS data ends at the location pointed to by the thread pointer.
pub(super) const TLS_OFFSET: usize = 0;

/// `munmap` the current thread, then carefully exit the thread without
/// touching the deallocated stack.
#[inline]
pub(super) unsafe fn munmap_and_exit_thread(map_addr: *mut c_void, map_len: usize) -> ! {
    asm!(
        // Use `int 0x80` instead of vsyscall, since vsyscall would attempt to
        // touch the stack after we `munmap` it.
        "int 0x80",
        "xor ebx,ebx",
        "mov eax,{__NR_exit}",
        "int 0x80",
        "ud2",
        __NR_exit = const __NR_exit,
        in("eax") __NR_munmap,
        in("ebx") map_addr,
        in("ecx") map_len,
        options(noreturn, nostack)
    );
}
