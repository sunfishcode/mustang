#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![feature(asm)]
#![feature(naked_functions)]
#![cfg_attr(debug_assertions, feature(link_llvm_intrinsics))]

/// The program entry point.
///
/// # Safety
///
/// This function should never be called explicitly. It is the first thing
/// executed in the program, and it assumes that memory is layed out
/// according to the operating system convention for starting a new program.
#[naked]
#[no_mangle]
unsafe extern "C" fn _start() -> ! {
    #[cfg(target_arch = "x86_64")]
    asm!("mov rdi, rsp",
         "push rbp",
         "jmp {start_rust}",
         start_rust = sym start_rust,
         options(noreturn));

    #[cfg(target_arch = "aarch64")]
    asm!("mov x0, sp",
         "mov x30, xzr",
         "b {start_rust}",
         start_rust = sym start_rust,
         options(noreturn));

    #[cfg(target_arch = "riscv64")]
    asm!("mv a0, sp",
         "mv ra, zero",
         "mv fp, zero",
         "j {start_rust}",
         start_rust = sym start_rust,
         options(noreturn));

    #[cfg(target_arch = "x86")]
    asm!("mov eax, esp",
         "push ebp",
         "push ebp",
         "push ebp",
         "push eax",
         "push ebp",
         "jmp {start_rust}",
         start_rust = sym start_rust,
         options(noreturn));
}

/// Rust entry point.
///
/// # Safety
///
/// `mem` should point to the stack as provided by the operating system.
unsafe extern "C" fn start_rust(mem: *mut usize) -> ! {
    use std::os::raw::{c_char, c_int, c_void};
    #[cfg(debug_assertions)]
    extern "C" {
        #[link_name = "llvm.frameaddress"]
        fn builtin_frame_address(level: i32) -> *const u8;
        #[link_name = "llvm.returnaddress"]
        fn builtin_return_address(level: i32) -> *const u8;
        #[cfg(target_arch = "aarch64")]
        #[link_name = "llvm.sponentry"]
        fn builtin_sponentry() -> *const u8;
    }
    extern "C" {
        fn main(argc: c_int, argv: *mut *mut c_char, envp: *mut *mut c_char) -> c_int;
        static __init_array_start: c_void;
        static __init_array_end: c_void;
    }

    // Do some basic invariant checks, to ensure that our assembly code did
    // what we expect it to do. These are debug-only for now, to keep the
    // release-mode startup code simple to disassemble and inspect, while we're
    // getting started.
    #[cfg(debug_assertions)]
    {
        debug_assert_ne!(mem, std::ptr::null_mut());
        debug_assert_eq!(mem as usize & 0xf, 0);
        debug_assert_eq!(builtin_return_address(0), std::ptr::null());
        debug_assert_ne!(builtin_frame_address(0), std::ptr::null());
        #[cfg(not(target_arch = "x86"))]
        debug_assert_eq!(builtin_frame_address(0) as usize & 0xf, 0);
        #[cfg(target_arch = "x86")]
        debug_assert_eq!(builtin_frame_address(0) as usize & 0xf, 8);
        debug_assert!((builtin_frame_address(0) as usize) <= mem as usize);
        debug_assert_eq!(builtin_frame_address(1), std::ptr::null());
        #[cfg(target_arch = "aarch64")]
        debug_assert_ne!(builtin_sponentry(), std::ptr::null());
        #[cfg(target_arch = "aarch64")]
        debug_assert_eq!(builtin_sponentry() as usize & 0xf, 0);
    }

    // For now, print a message, so that we know we're doing something. We'll
    // probably remove this at some point, but for now, things are fragile
    // enough that it's nice to have this confirmation.
    #[cfg(debug_assertions)]
    eprintln!("This process was started by mustang! 🐎");

    // Compute `argc`, `argv`, and `envp`.
    let argc = *mem as c_int;
    let argv = mem.add(1) as *mut *mut c_char;
    let envp = argv.add(argc as usize + 1);

    // Do a few more invariant checks on `argc` and `argv`.
    debug_assert!(argc >= 0);
    debug_assert!((*argv.add(argc as usize)).is_null());

    // Call the `.init_array` functions.
    {
        type InitFn = fn(c_int, *mut *mut c_char, *mut *mut c_char);
        let mut init = &__init_array_start as *const _ as usize as *const InitFn;
        let init_end = &__init_array_end as *const _ as usize as *const InitFn;
        while init != init_end {
            (*init)(argc, argv, envp);
            init = init.add(1);
        }
    }

    // Call `main`.
    let ret = main(argc, argv, envp);

    // Exit with main's return value. For now, we use `std`, which takes care
    // of `.fini_array`, and `atexit` functions.
    std::process::exit(ret)
}
