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
extern crate c;
#[cfg(target_vendor = "mustang")]
extern crate origin;

#[cfg(target_vendor = "mustang")]
#[cfg(feature = "dlmalloc")]
#[global_allocator]
static GLOBAL_ALLOCATOR: dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;

#[cfg(target_vendor = "mustang")]
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static GLOBAL_ALLOCATOR: wee_alloc::WeeAlloc<'static> = wee_alloc::WeeAlloc::INIT;

// We need to enable some global allocator, because `c-scape` implements
// `malloc` using the Rust global allocator, and the default Rust global
// allocator uses `malloc`.
#[cfg(target_vendor = "mustang")]
#[cfg(not(any(feature = "dlmalloc", feature = "wee_alloc")))]
compile_error!("Either feature \"dlmalloc\" or \"wee_alloc\" must be enabled for this crate.");
