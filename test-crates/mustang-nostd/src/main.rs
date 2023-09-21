#![allow(internal_features)]
#![feature(lang_items)]
#![no_std]
// When testing we do not want to use our main function
#![cfg_attr(not(test), no_main)]

mustang::can_run_this!();

use core::ffi::c_int;

#[cfg(not(test))]
#[panic_handler]
fn panic(_panic: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}

#[cfg(not(test))]
#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

#[global_allocator]
static GLOBAL_ALLOCATOR: rustix_dlmalloc::GlobalDlmalloc = rustix_dlmalloc::GlobalDlmalloc;

// Small hack for rust-analyzer.
//
// If we do `#[cfg(not(test))]` then rust-analyzer will say the code is inactive and we
// lose the ability to use rust-analyzer.
// By disabling `no_mangle` during test, we do not get the two main functions defined error.
// This function ends up just behaving like any other (unused) function in the binary during
// test compilation.
//
// Using `#[start]` would not require us to use this hack but then we lose `_envp`
// function parameter.
#[cfg_attr(not(test), no_mangle)]
extern "C" fn main(_argc: c_int, _argv: *mut *mut u8, _envp: *mut *mut u8) -> c_int {
    0
}

#[test]
fn test_tests() {
    assert_eq!(1, 1)
}
