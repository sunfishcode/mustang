#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![feature(asm)]
#![feature(naked_functions)]
#![feature(link_llvm_intrinsics)]
#![feature(atomic_mut_ptr)]
#![cfg(target_vendor = "mustang")]

mod entry;
mod process;
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

use entry::program;
use std::ffi::c_void;
#[cfg(feature = "initialize-c-runtime")]
use std::os::raw::{c_char, c_int};

pub use process::{at_exit, exit, exit_immediately};
#[cfg(feature = "threads")]
pub use threads::{
    at_thread_exit, create_thread, current_thread, current_thread_id, default_guard_size,
    default_stack_size, detach_thread, join_thread, Thread,
};

/// The program entry point.
///
/// # Safety
///
/// This function should never be called explicitly. It is the first thing
/// executed in the program, and it assumes that memory is laid out
/// according to the operating system convention for starting a new program.
#[naked]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn _start() -> ! {
    // Jump to `program`, passing it the initial stack pointer value as an
    // argument, a NULL return address, a NULL frame pointer, and an aligned
    // stack pointer.

    #[cfg(target_arch = "x86_64")]
    asm!("mov rdi, rsp",
         "push rbp",
         "jmp {program}",
         program = sym program,
         options(noreturn));

    #[cfg(target_arch = "aarch64")]
    asm!("mov x0, sp",
         "mov x30, xzr",
         "b {program}",
         program = sym program,
         options(noreturn));

    #[cfg(target_arch = "riscv64")]
    asm!("mv a0, sp",
         "mv ra, zero",
         "mv fp, zero",
         "tail {program}",
         program = sym program,
         options(noreturn));

    #[cfg(target_arch = "x86")]
    asm!("mov eax, esp",
         "push ebp",
         "push ebp",
         "push ebp",
         "push eax",
         "push ebp",
         "jmp {program}",
         program = sym program,
         options(noreturn));
}

#[repr(transparent)]
struct SendSyncVoidStar(*mut c_void);

unsafe impl Send for SendSyncVoidStar {}
unsafe impl Sync for SendSyncVoidStar {}

/// An ABI-conforming `__dso_handle`.
#[link_section = ".data.__mustang"]
#[no_mangle]
#[used]
static __dso_handle: SendSyncVoidStar = SendSyncVoidStar(&__dso_handle as *const _ as *mut c_void);

#[cfg(feature = "initialize-c-runtime")]
unsafe fn initialize_c_runtime(argc: c_int, argv: *mut *mut c_char) {
    #[link(name = "initialize-c-runtime")]
    extern "C" {
        fn mustang_initialize_c_runtime(argc: c_int, argv: *mut *mut c_char);
    }

    #[cfg(debug_assertions)]
    eprintln!(".｡oO(C runtime initialization called by origin! ℂ)");

    mustang_initialize_c_runtime(argc, argv);
}

/// Ensure that this module is linked in.
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
#[cold]
unsafe extern "C" fn __mustang_origin() {
    asm!("# {}", in(reg) &__dso_handle);
}
