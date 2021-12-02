#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![feature(asm)]
#![feature(asm_const)]
#![feature(asm_sym)]
#![feature(naked_functions)]
#![feature(link_llvm_intrinsics)]
#![feature(atomic_mut_ptr)]
#![feature(const_fn_fn_ptr_basics)]
#![no_std]

extern crate alloc;

mod allocator;
mod mutex;
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
#[cfg(all(target_arch = "arm", feature = "threads"))]
#[path = "arch-arm.rs"]
mod arch;

use core::ffi::c_void;

pub use program::{at_exit, at_fork, exit, exit_immediately, fork};
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
#[no_mangle]
unsafe extern "C" fn _start() -> ! {
    use program::entry;

    // Jump to `entry`, passing it the initial stack pointer value as an
    // argument, a NULL return address, a NULL frame pointer, and an aligned
    // stack pointer. On many architectures, the incoming frame pointer is
    // already NULL.

    #[cfg(target_arch = "x86_64")]
    asm!("mov rdi, rsp",   // Pass the incoming `rsp` as the arg to `entry`.
         "push rbp",       // Set the return address to zero.
         "jmp {entry}",    // Jump to `entry`.
         entry = sym entry,
         options(noreturn));

    #[cfg(target_arch = "aarch64")]
    asm!("mov x0, sp",     // Pass the incoming `sp` as the arg to `entry`.
         "mov x30, xzr",   // Set the return address to zero.
         "b {entry}",      // Jump to `entry`.
         entry = sym entry,
         options(noreturn));

    #[cfg(target_arch = "arm")]
    asm!("mov r0, sp\n",   // Pass the incoming `sp` as the arg to `entry`.
         "mov lr, #0",     // Set the return address to zero.
         "b {entry}",      // Jump to `entry`.
         entry = sym entry,
         options(noreturn));

    #[cfg(target_arch = "riscv64")]
    asm!("mv a0, sp",      // Pass the incoming `sp` as the arg to `entry`.
         "mv ra, zero",    // Set the return address to zero.
         "mv fp, zero",    // Set the frame address to zero.
         "tail {entry}",   // Jump to `entry`.
         entry = sym entry,
         options(noreturn));

    #[cfg(target_arch = "x86")]
    asm!("mov eax, esp",   // Save the incoming `esp` value.
         "push ebp",       // Pad for alignment.
         "push ebp",       // Pad for alignment.
         "push ebp",       // Pad for alignment.
         "push eax",       // Pass saved the incoming `esp` as the arg to `entry`.
         "push ebp",       // Set the return address to zero.
         "jmp {entry}",    // Jump to `entry`.
         entry = sym entry,
         options(noreturn));
}

#[repr(transparent)]
struct SendSyncVoidStar(*mut c_void);

unsafe impl Send for SendSyncVoidStar {}
unsafe impl Sync for SendSyncVoidStar {}

/// An ABI-conforming `__dso_handle`.
#[cfg(target_vendor = "mustang")]
#[no_mangle]
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

#[cfg(target_vendor = "mustang")]
#[global_allocator]
static GLOBAL_ALLOCATOR: allocator::GlobalAllocator = allocator::GlobalAllocator;
