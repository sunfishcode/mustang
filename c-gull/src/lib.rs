#![doc = include_str!("../README.md")]
#![no_builtins] // don't let LLVM optimize our `memcpy` into a `memcpy` call
#![feature(strict_provenance)]
#![feature(c_variadic)]
#![feature(sync_unsafe_cell)]
#![deny(fuzzy_provenance_casts)]
#![deny(lossy_provenance_casts)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate c_scape;

// Re-export the libc crate's API. This allows users to depend on the c-scape
// crate in place of libc.
pub use libc::*;

#[macro_use]
mod use_libc;

#[cfg(feature = "std")]
mod nss;
#[cfg(feature = "std")]
mod printf;
#[cfg(feature = "std")]
mod resolve;
#[cfg(feature = "std")]
mod strtod;
#[cfg(feature = "std")]
mod strtol;
#[cfg(feature = "std")]
mod sysconf;
#[cfg(feature = "std")]
mod termios;
#[cfg(feature = "std")]
mod time;
#[cfg(feature = "std")]
mod utmp;

#[cfg(feature = "std")]
#[no_mangle]
unsafe extern "C" fn __assert_fail(
    expr: *const c_char,
    file: *const c_char,
    line: c_int,
    func: *const c_char,
) -> ! {
    use std::ffi::CStr;
    //libc!(libc::__assert_fail(expr, file, line, func));

    eprintln!(
        "Assertion failed: {:?} ({:?}:{}: {:?})",
        CStr::from_ptr(expr),
        CStr::from_ptr(file),
        line,
        CStr::from_ptr(func)
    );
    std::process::abort();
}

// utilities

/// Convert a rustix `Result` into an `Option` with the error stored
/// in `errno`.
#[cfg(feature = "std")]
fn convert_res<T>(result: Result<T, rustix::io::Errno>) -> Option<T> {
    use errno::{set_errno, Errno};
    result
        .map_err(|err| set_errno(Errno(err.raw_os_error())))
        .ok()
}
