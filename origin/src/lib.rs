#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![cfg_attr(target_vendor = "mustang", feature(asm))]
#![cfg_attr(target_vendor = "mustang", feature(naked_functions))]
#![cfg_attr(
    all(target_vendor = "mustang", debug_assertions),
    feature(link_llvm_intrinsics)
)]

#[cfg(target_vendor = "mustang")]
use std::os::raw::{c_char, c_int, c_void};

/// The program entry point.
///
/// # Safety
///
/// This function should never be called explicitly. It is the first thing
/// executed in the program, and it assumes that memory is laid out
/// according to the operating system convention for starting a new program.
#[cfg(target_vendor = "mustang")]
#[naked]
#[no_mangle]
unsafe extern "C" fn _start() -> ! {
    // Jump to `rust`, passing it the initial stack pointer value as an
    // argument, a NULL return address, a NULL frame pointer, and an aligned
    // stack pointer.

    #[cfg(target_arch = "x86_64")]
    asm!("mov rdi, rsp",
         "push rbp",
         "jmp {rust}",
         rust = sym rust,
         options(noreturn));

    #[cfg(target_arch = "aarch64")]
    asm!("mov x0, sp",
         "mov x30, xzr",
         "b {rust}",
         rust = sym rust,
         options(noreturn));

    #[cfg(target_arch = "riscv64")]
    asm!("mv a0, sp",
         "mv ra, zero",
         "mv fp, zero",
         "tail {rust}",
         rust = sym rust,
         options(noreturn));

    #[cfg(target_arch = "x86")]
    asm!("mov eax, esp",
         "push ebp",
         "push ebp",
         "push ebp",
         "push eax",
         "push ebp",
         "jmp {rust}",
         rust = sym rust,
         options(noreturn));
}

/// Rust entry point.
///
/// # Safety
///
/// `mem` should point to the stack as provided by the operating system.
#[cfg(target_vendor = "mustang")]
unsafe extern "C" fn rust(mem: *mut usize) -> ! {
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
        fn exit(code: c_int) -> !;
        static __init_array_start: c_void;
        static __init_array_end: c_void;
    }

    // Do some basic precondition checks, to ensure that our assembly code did
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
    eprintln!(".｡oO(This process was started by origin! 🎯)");

    // Compute `argc`, `argv`, and `envp`.
    let argc = *mem as c_int;
    let argv = mem.add(1).cast::<*mut c_char>();
    let envp = argv.add(argc as usize + 1);

    // Do a few more precondition checks on `argc` and `argv`.
    debug_assert!(argc >= 0);
    debug_assert_eq!(*mem, argc as _);
    debug_assert_eq!(*argv.add(argc as usize), std::ptr::null_mut());

    // Call the `.init_array` functions (unless the C runtime is doing it;
    // see below).
    #[cfg(not(feature = "initialize-c-runtime"))]
    {
        type InitFn = fn(c_int, *mut *mut c_char, *mut *mut c_char);
        let mut init = &__init_array_start as *const _ as usize as *const InitFn;
        let init_end = &__init_array_end as *const _ as usize as *const InitFn;
        // Prevent the optimizer from optimizing the `!=` comparison to true;
        // `fini` and `fini_start` may have the same address.
        asm!("# {}", inout(reg) init);
        while init != init_end {
            (*init)(argc, argv, envp);
            init = init.add(1);
        }
    }

    // If enabled, initialize the C runtime. This also handles calling the
    // .init_array functions.
    #[cfg(feature = "initialize-c-runtime")]
    initialize_c_runtime(argc, argv);

    // Call `main`.
    let ret = main(argc, argv, envp);

    // Exit with main's return value. This could be a libc `exit`, or our
    // own `c-scape`'s `exit`. This takes care of the `.fini_array`,
    // `__cxa_atexit`, and `atexit` functions.
    exit(ret)
}

#[cfg(target_vendor = "mustang")]
#[repr(transparent)]
struct SendSyncVoidStar(*mut c_void);

#[cfg(target_vendor = "mustang")]
unsafe impl Send for SendSyncVoidStar {}

#[cfg(target_vendor = "mustang")]
unsafe impl Sync for SendSyncVoidStar {}

/// An ABI-conforming `__dso_handle`.
#[cfg(target_vendor = "mustang")]
#[no_mangle]
#[used]
static __dso_handle: SendSyncVoidStar = SendSyncVoidStar(&__dso_handle as *const _ as *mut c_void);

#[cfg(target_vendor = "mustang")]
#[cfg(feature = "initialize-c-runtime")]
unsafe fn initialize_c_runtime(argc: c_int, argv: *mut *mut c_char) {
    #[link(name = "initialize-c-runtime")]
    extern "C" {
        fn mustang_initialize_c_runtime(argc: c_int, argv: *mut *mut c_char);
    }

    #[cfg(debug_assertions)]
    eprintln!(".｡oO(C runtime initialization called by origin! ℂ)");

    mustang_initialize_c_runtime(argc, argv);
}
