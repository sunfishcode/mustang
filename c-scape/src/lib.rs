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

#[cfg(target_vendor = "mustang")]
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
#[cfg(target_vendor = "mustang")]
mod signal;
mod termios_;

#[cfg(feature = "threads")]
#[cfg(target_vendor = "mustang")]
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

// utilities

/// Convert a rustix `Result` into an `Option` with the error stored
/// in `errno`.
fn convert_res<T>(result: Result<T, rustix::io::Errno>) -> Option<T> {
    result
        .map_err(|err| set_errno(Errno(err.raw_os_error())))
        .ok()
}
