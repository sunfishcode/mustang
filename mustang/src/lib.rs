#![doc = include_str!("../README.md")]

/// Declare that a program can be compiled and run by `mustang`.
///
/// To use this, put `mustang::can_run_this!()` in the top-level `main.rs`. In
/// `*-mustang-*` builds, this arranges for `mustang` libraries to be used. In
/// all other builds, this does nothing.
#[macro_export]
macro_rules! can_run_this {
    () => {};
}

#[cfg(target_vendor = "mustang")]
extern crate c_scape;
#[cfg(target_vendor = "mustang")]
extern crate origin;
#[cfg(not(target_arch = "arm"))]
#[cfg(target_vendor = "mustang")]
extern crate unwinding;
