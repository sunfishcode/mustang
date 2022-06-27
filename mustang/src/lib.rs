#![doc = include_str!("../README.md")]
// Enable strict-provenance APIs and lints. We only do this for mustang to
// prevent non-mustang compilations from depending on nightly Rust.
#![cfg_attr(target_vendor = "mustang", feature(strict_provenance))]
#![cfg_attr(target_vendor = "mustang", deny(fuzzy_provenance_casts))]
#![cfg_attr(target_vendor = "mustang", deny(lossy_provenance_casts))]

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
extern crate c_scape;
#[cfg(target_vendor = "mustang")]
extern crate origin;
#[cfg(not(target_arch = "arm"))]
#[cfg(target_vendor = "mustang")]
extern crate unwinding;

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
