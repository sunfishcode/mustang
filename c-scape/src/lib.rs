#![doc = include_str!("../README.md")]
#![no_builtins] // don't let LLVM optimize our `memcpy` into a `memcpy` call
#![feature(asm)]
#![feature(c_variadic)]
#![cfg(target_vendor = "mustang")]

#[cfg(mustang_use_libc)]
#[macro_use]
mod use_libc;

mod c;
mod data;
mod environ;
mod error_str;
mod exit;
mod m;
mod pthread;
mod unwind;

/// Link in `wee_alloc` as the global allocator, so that we don't call
/// `malloc`.
type GlobalAlloc = wee_alloc::WeeAlloc<'static>;
const GLOBAL_ALLOC: GlobalAlloc = wee_alloc::WeeAlloc::INIT;
#[global_allocator]
static ALLOC: crate::GlobalAlloc = crate::GLOBAL_ALLOC;

/// Ensure that `mustang`'s modules are linked in.
#[inline(never)]
#[link_section = ".mustang"]
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
    link!(__mustang_c_scape__pthread);
    link!(__mustang_c_scape__exit);
    link!(__mustang_c_scape__unwind);
    link!(__mustang_c_scape__environ);
}
