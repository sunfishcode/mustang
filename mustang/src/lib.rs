#![doc = include_str!("../README.md")]

extern crate c_scape;
extern crate origin;

type GlobalAlloc = wee_alloc::WeeAlloc<'static>;

const GLOBAL_ALLOC: GlobalAlloc = wee_alloc::WeeAlloc::INIT;

#[global_allocator]
static ALLOC: crate::GlobalAlloc = crate::GLOBAL_ALLOC;
