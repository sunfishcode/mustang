#![doc = include_str!("../README.md")]
#![no_std]

/// Declare that a program can be compiled and run by `mustang`.
///
/// To use this, put `mustang::can_run_this!()` in the top-level `main.rs`. In
/// `*-mustang-*` builds, this arranges for `mustang` libraries to be used. In
/// all other builds, this does nothing.
#[macro_export]
macro_rules! can_run_this {
    // This expands to nothing, yet appears to be sufficient.
    //
    // If the user just declares mustang as a dependency in Cargo.toml, rustc
    // appears to notice that it's not actually used, and doesn't include it,
    // so we don't get a chance to override libc and register our global
    // allocator.
    //
    // It appears that just this empty macro is sufficient to make rustc
    // believe that the crate is used, which gives us the chance we need.
    () => {};
}

#[cfg(target_vendor = "mustang")]
extern crate c_gull;
