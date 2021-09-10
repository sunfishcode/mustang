#![doc = include_str!("../README.md")]
#![no_builtins] // don't let LLVM optimize our `memcpy` into a `memcpy` call
#![feature(asm)]
#![feature(c_variadic)]

#[cfg(mustang_use_libc)]
#[macro_use]
mod use_libc;

mod c;
mod data;
mod environ;
mod error_str;
mod exit;
mod unwind;

type GlobalAlloc = wee_alloc::WeeAlloc<'static>;

const GLOBAL_ALLOC: GlobalAlloc = wee_alloc::WeeAlloc::INIT;

#[global_allocator]
static ALLOC: crate::GlobalAlloc = crate::GLOBAL_ALLOC;
