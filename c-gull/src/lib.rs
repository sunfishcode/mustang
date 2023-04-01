#![doc = include_str!("../README.md")]
#![no_builtins] // don't let LLVM optimize our `memcpy` into a `memcpy` call
#![feature(strict_provenance)]
#![feature(c_variadic)]
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
mod time;

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
