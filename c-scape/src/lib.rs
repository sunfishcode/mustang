#![doc = include_str!("../README.md")]
#![no_std]
#![no_builtins] // don't let LLVM optimize our `memcpy` into a `memcpy` call
#![feature(thread_local)] // for `__errno_location`
#![feature(c_variadic)] // for `ioctl` etc.
#![feature(rustc_private)] // for compiler-builtins
#![feature(strict_provenance)]
#![feature(inline_const)]
#![feature(sync_unsafe_cell)]
#![deny(fuzzy_provenance_casts)]
#![deny(lossy_provenance_casts)]
#![feature(try_blocks)]

// Check that our features were used as we intend.
#[cfg(all(feature = "coexist-with-libc", feature = "take-charge"))]
compile_error!("Enable only one of \"coexist-with-libc\" and \"take-charge\".");
#[cfg(all(not(feature = "coexist-with-libc"), not(feature = "take-charge")))]
compile_error!("Enable one \"coexist-with-libc\" and \"take-charge\".");

extern crate alloc;
extern crate compiler_builtins;

// Re-export the libc crate's API. This allows users to depend on the c-scape
// crate in place of libc.
pub use libc::*;

#[macro_use]
mod use_libc;

#[cfg(not(target_os = "wasi"))]
mod at_fork;
mod error_str;
mod sync_ptr;

// Selected libc-compatible interfaces.
//
// The goal here isn't necessarily to build a complete libc; it's primarily
// to provide things that `std` and possibly popular crates are currently
// using.
//
// This effectively undoes the work that `rustix` does: it calls `rustix` and
// translates it back into a C-like ABI. Ideally, Rust code should just call
// the `rustix` APIs directly, which are safer, more ergonomic, and skip this
// whole layer.

use errno::{set_errno, Errno};

mod ctype;
mod env;
mod fs;
mod io;

#[cfg(feature = "take-charge")]
mod malloc;

mod math;
mod mem;

#[cfg(not(target_os = "wasi"))]
mod mmap;

mod net;

#[cfg(not(target_os = "wasi"))]
mod process;

mod rand;
mod rand48;
#[cfg(not(target_os = "wasi"))]
#[cfg(feature = "take-charge")]
mod signal;
mod termios_;

#[cfg(feature = "threads")]
#[cfg(feature = "take-charge")]
mod threads;

mod errno_;
mod exec;
mod exit;
mod glibc_versioning;
mod nss;
mod posix_spawn;
mod process_;
mod rand_;
mod setjmp;
mod syscall;
mod time;

/// An ABI-conforming `__dso_handle`.
#[cfg(feature = "take-charge")]
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
#[repr(transparent)]
#[cfg(feature = "take-charge")]
struct UnsafeSendSyncVoidStar(*const core::ffi::c_void);
#[cfg(feature = "take-charge")]
unsafe impl Send for UnsafeSendSyncVoidStar {}
#[cfg(feature = "take-charge")]
unsafe impl Sync for UnsafeSendSyncVoidStar {}

/// Adapt from origin's `origin_main` to a C ABI `main`.
#[no_mangle]
fn origin_main(argc: usize, argv: *mut *mut u8, envp: *mut *mut u8) -> i32 {
    extern "C" {
        fn main(argc: i32, argv: *const *const u8, envp: *const *const u8) -> i32;
    }
    unsafe { main(argc as _, argv as _, envp as _) }
}

// utilities

/// Convert a rustix `Result` into an `Option` with the error stored
/// in `errno`.
fn convert_res<T>(result: Result<T, rustix::io::Errno>) -> Option<T> {
    result
        .map_err(|err| set_errno(Errno(err.raw_os_error())))
        .ok()
}
