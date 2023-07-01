#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![feature(asm_const)]
#![feature(naked_functions)]
#![feature(link_llvm_intrinsics)]
#![feature(strict_provenance)]
#![deny(fuzzy_provenance_casts)]
#![deny(lossy_provenance_casts)]
#![no_std]

#[cfg(not(feature = "rustc-dep-of-std"))]
extern crate alloc;

pub mod sync;

mod program;
#[cfg(target_vendor = "mustang")]
mod signal;
#[cfg(feature = "threads")]
#[cfg_attr(not(target_vendor = "mustang"), path = "threads_via_pthreads.rs")]
mod threads;

#[cfg(feature = "threads")]
#[cfg(target_vendor = "mustang")]
#[cfg_attr(target_arch = "aarch64", path = "arch-aarch64.rs")]
#[cfg_attr(target_arch = "x86_64", path = "arch-x86_64.rs")]
#[cfg_attr(target_arch = "x86", path = "arch-x86.rs")]
#[cfg_attr(target_arch = "riscv64", path = "arch-riscv64.rs")]
#[cfg_attr(target_arch = "arm", path = "arch-arm.rs")]
mod arch;

pub use program::{at_exit, exit, exit_immediately};
#[cfg(target_vendor = "mustang")]
pub use signal::{sigaction, Sigaction};
#[cfg(feature = "threads")]
#[cfg(feature = "set_thread_id")]
pub use threads::set_current_thread_id_after_a_fork;
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
/// executed in the program, and it assumes that memory is laid out according
/// to the operating system convention for starting a new program.
#[cfg(target_vendor = "mustang")]
#[naked]
#[no_mangle]
unsafe extern "C" fn _start() -> ! {
    use core::arch::asm;
    use program::entry;

    // Jump to `entry`, passing it the initial stack pointer value as an
    // argument, a null return address, a null frame pointer, and an aligned
    // stack pointer. On many architectures, the incoming frame pointer is
    // already null.

    #[cfg(target_arch = "x86_64")]
    asm!(
        "mov rdi, rsp", // Pass the incoming `rsp` as the arg to `entry`.
        "push rbp",     // Set the return address to zero.
        "jmp {entry}",  // Jump to `entry`.
        entry = sym entry,
        options(noreturn),
    );

    #[cfg(target_arch = "aarch64")]
    asm!(
        "mov x0, sp",   // Pass the incoming `sp` as the arg to `entry`.
        "mov x30, xzr", // Set the return address to zero.
        "b {entry}",    // Jump to `entry`.
        entry = sym entry,
        options(noreturn),
    );

    #[cfg(target_arch = "arm")]
    asm!(
        "mov r0, sp\n", // Pass the incoming `sp` as the arg to `entry`.
        "mov lr, #0",   // Set the return address to zero.
        "b {entry}",    // Jump to `entry`.
        entry = sym entry,
        options(noreturn),
    );

    #[cfg(target_arch = "riscv64")]
    asm!(
        "mv a0, sp",    // Pass the incoming `sp` as the arg to `entry`.
        "mv ra, zero",  // Set the return address to zero.
        "mv fp, zero",  // Set the frame address to zero.
        "tail {entry}", // Jump to `entry`.
        entry = sym entry,
        options(noreturn),
    );

    #[cfg(target_arch = "x86")]
    asm!(
        "mov eax, esp", // Save the incoming `esp` value.
        "push ebp",     // Pad for alignment.
        "push ebp",     // Pad for alignment.
        "push ebp",     // Pad for alignment.
        "push eax",     // Pass saved the incoming `esp` as the arg to `entry`.
        "push ebp",     // Set the return address to zero.
        "jmp {entry}",  // Jump to `entry`.
        entry = sym entry,
        options(noreturn),
    );
}

/// An ABI-conforming `__dso_handle`.
#[cfg(target_vendor = "mustang")]
#[no_mangle]
static __dso_handle: UnsafeSendSyncVoidStar =
    UnsafeSendSyncVoidStar(&__dso_handle as *const _ as *const _);

/// A type for `__dso_handle`.
///
/// `*const c_void` isn't `Send` or `Sync` because a raw pointer could point to
/// arbitrary data which isn't thread-safe, however `__dso_handle` is used as
/// an opaque cookie value, and it always points to itself.
///
/// Note that in C, `__dso_handle`'s type is usually `void *` which would
/// correspond to `*mut c_void`, however we can assume the pointee is never
/// actually mutated.
#[cfg(target_vendor = "mustang")]
#[repr(transparent)]
struct UnsafeSendSyncVoidStar(*const core::ffi::c_void);
#[cfg(target_vendor = "mustang")]
unsafe impl Send for UnsafeSendSyncVoidStar {}
#[cfg(target_vendor = "mustang")]
unsafe impl Sync for UnsafeSendSyncVoidStar {}

/// Initialize logging, if enabled.
#[cfg(target_vendor = "mustang")]
#[cfg(feature = "log")]
#[link_section = ".init_array.00099"]
#[used]
static INIT_ARRAY: unsafe extern "C" fn() = {
    unsafe extern "C" fn function() {
        #[cfg(feature = "env_logger")]
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

// Re-export this so that our users can use the same version we do.
pub use lock_api;
