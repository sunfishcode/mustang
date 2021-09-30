#![doc = include_str!("../README.md")]
#![no_builtins] // don't let LLVM optimize our `memcpy` into a `memcpy` call
#![feature(thread_local)] // for `__errno_location`
#![feature(c_variadic)] // for `ioctl` etc.
#![feature(rustc_private)] // for compiler-builtins
#![feature(untagged_unions)] // for `pthread_mutex_t`
#![feature(atomic_mut_ptr)] // for `RawMutex`
#![cfg(target_vendor = "mustang")]

#[macro_use]
mod use_libc;

mod c;
mod data;
mod environ;
mod error_str;
mod exit;
mod m;
#[cfg(feature = "threads")]
mod pthread;
#[cfg(feature = "threads")]
mod raw_mutex;
mod unwind;

/// Link in `wee_alloc` as the global allocator, so that we don't call
/// `malloc`.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc<'static> = wee_alloc::WeeAlloc::INIT;

/// Ensure that `mustang`'s modules are linked in.
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
#[cold]
unsafe extern "C" fn __mustang_c_scape() {
    macro_rules! link {
        ($name:ident) => {{
            extern "C" {
                fn $name();
            }
            $name();
        }};
    }
    link!(__mustang_c_scape__c);
    link!(__mustang_c_scape__m);
    #[cfg(feature = "threads")]
    link!(__mustang_c_scape__pthread);
    link!(__mustang_c_scape__exit);
    link!(__mustang_c_scape__unwind);
    link!(__mustang_c_scape__environ);
}
