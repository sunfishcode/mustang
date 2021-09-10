#![doc = include_str!("../README.md")]
#![no_builtins] // don't let LLVM optimize our `memcpy` into a `memcpy` call
#![feature(asm)]
#![feature(c_variadic)]

mod c;
mod environ;
mod error_str;
mod exit;
mod pthread;
mod unwind;
