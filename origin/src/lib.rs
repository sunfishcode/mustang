#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![feature(asm)]
#![feature(naked_functions)]
#![feature(link_llvm_intrinsics)]
#![feature(atomic_mut_ptr)]

mod program;
#[cfg(feature = "threads")]
mod threads;

#[cfg(all(target_arch = "aarch64", feature = "threads"))]
#[path = "arch-aarch64.rs"]
mod arch;
#[cfg(all(target_arch = "x86_64", feature = "threads"))]
#[path = "arch-x86_64.rs"]
mod arch;
#[cfg(all(target_arch = "x86", feature = "threads"))]
#[path = "arch-x86.rs"]
mod arch;
#[cfg(all(target_arch = "riscv64", feature = "threads"))]
#[path = "arch-riscv64.rs"]
mod arch;

use std::ffi::c_void;

pub use program::{at_exit, exit, exit_immediately};
#[cfg(feature = "threads")]
pub use threads::{
    at_thread_exit, create_thread, current_thread, current_thread_id, current_thread_tls_addr,
    default_guard_size, default_stack_size, detach_thread, join_thread, thread_stack, Thread,
};

/// The program entry point.
///
/// # Safety
///
/// This function should never be called explicitly. It is the first thing
/// executed in the program, and it assumes that memory is laid out
/// according to the operating system convention for starting a new program.
#[cfg(target_vendor = "mustang")]
#[naked]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn _start() -> ! {
    use program::entry;

    // Jump to `program`, passing it the initial stack pointer value as an
    // argument, a NULL return address, a NULL frame pointer, and an aligned
    // stack pointer.

    #[cfg(target_arch = "x86_64")]
    asm!("mov rdi, rsp",
         "push rbp",
         "jmp {entry}",
         entry = sym entry,
         options(noreturn));

    #[cfg(target_arch = "aarch64")]
    asm!("mov x0, sp",
         "mov x30, xzr",
         "b {entry}",
         entry = sym entry,
         options(noreturn));

    #[cfg(target_arch = "riscv64")]
    asm!("mv a0, sp",
         "mv ra, zero",
         "mv fp, zero",
         "tail {entry}",
         entry = sym entry,
         options(noreturn));

    #[cfg(target_arch = "x86")]
    asm!("mov eax, esp",
         "push ebp",
         "push ebp",
         "push ebp",
         "push eax",
         "push ebp",
         "jmp {entry}",
         entry = sym entry,
         options(noreturn));
}

#[repr(transparent)]
struct SendSyncVoidStar(*mut c_void);

unsafe impl Send for SendSyncVoidStar {}
unsafe impl Sync for SendSyncVoidStar {}

/// An ABI-conforming `__dso_handle`.
#[cfg(target_vendor = "mustang")]
#[link_section = ".data.__mustang"]
#[no_mangle]
#[used]
static __dso_handle: SendSyncVoidStar = SendSyncVoidStar(&__dso_handle as *const _ as *mut c_void);

/// Initialize logging, if enabled.
#[cfg(target_vendor = "mustang")]
#[cfg(feature = "env_logger")]
#[link_section = ".init_array.00099"]
#[used]
static INIT_ARRAY: unsafe extern "C" fn() = {
    unsafe extern "C" fn function() {
        env_logger::init();

        log::trace!(target: "origin::program", "Program started");

        // Log the thread id. We initialized the main earlier than this, but
        // we couldn't initialize the logger until after the main thread is
        // intialized :-).
        #[cfg(feature = "threads")]
        log::trace!(target: "origin::threads", "Main Thread[{:?}] initialized", current_thread_id());
    }
    function
};

/// Ensure that this module is linked in.
#[cfg(target_vendor = "mustang")]
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
#[cold]
unsafe extern "C" fn __mustang_origin() {
    asm!("# {}", in(reg) &__dso_handle);
}
