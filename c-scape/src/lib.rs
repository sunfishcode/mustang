#![doc = include_str!("../README.md")]
#![no_builtins] // don't let LLVM optimize our `memcpy` into a `memcpy` call
#![feature(asm)]

mod c;
mod environ;
mod exit;
mod pthread;
mod unwind;
