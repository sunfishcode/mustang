#![doc = include_str!("../README.md")]

/// Declare that a program can be compiled by `mustang`.
///
/// To use this, put `mustang::can_compile_this!()` in the top-level
/// `main.rs`. In `*-mustang-*` builds, this arranges for `mustang`
/// libraries to be used. In all other builds, this does nothing.
#[macro_export]
macro_rules! can_compile_this {
    () => {};
}

#[cfg(target_vendor = "mustang")]
extern crate c;
#[cfg(target_vendor = "mustang")]
#[cfg(feature = "initialize_c_runtime")]
extern crate initialize_c_runtime;
#[cfg(target_vendor = "mustang")]
extern crate origin;
