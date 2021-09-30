//! Entrypoints where Rust code is entered from assembly code.

#[cfg(feature = "initialize-c-runtime")]
use crate::initialize_c_runtime;
#[cfg(not(feature = "initialize-c-runtime"))]
use crate::process::call_ctors;
use crate::process::exit;
#[cfg(feature = "threads")]
use crate::threads::exit_thread;
#[cfg(feature = "threads")]
use std::ffi::c_void;
use std::os::raw::{c_char, c_int};

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

/// The entrypoint where Rust code is first executed when the program starts.
///
/// # Safety
///
/// `mem` should point to the stack as provided by the operating system.
pub(super) unsafe extern "C" fn program(mem: *mut usize) -> ! {
    extern "C" {
        fn main(argc: c_int, argv: *mut *mut c_char, envp: *mut *mut c_char) -> c_int;
    }

    // Do some basic precondition checks, to ensure that our assembly code did
    // what we expect it to do. These are debug-only for now, to keep the
    // release-mode startup code simple to disassemble and inspect, while we're
    // getting started.
    #[cfg(debug_assertions)]
    {
        // Check that `mem` is where we expect it to be.
        debug_assert_ne!(mem, std::ptr::null_mut());
        debug_assert_eq!(mem as usize & 0xf, 0);
        debug_assert!((builtin_frame_address(0) as usize) <= mem as usize);

        // Check that the incoming stack pointer is where we expect it to be.
        debug_assert_eq!(builtin_return_address(0), std::ptr::null());
        debug_assert_ne!(builtin_frame_address(0), std::ptr::null());
        #[cfg(not(target_arch = "x86"))]
        debug_assert_eq!(builtin_frame_address(0) as usize & 0xf, 0);
        #[cfg(target_arch = "x86")]
        debug_assert_eq!(builtin_frame_address(0) as usize & 0xf, 8);
        debug_assert_eq!(builtin_frame_address(1), std::ptr::null());
        #[cfg(target_arch = "aarch64")]
        debug_assert_ne!(builtin_sponentry(), std::ptr::null());
        #[cfg(target_arch = "aarch64")]
        debug_assert_eq!(builtin_sponentry() as usize & 0xf, 0);
    }

    // Compute `argc`, `argv`, and `envp`.
    let argc = *mem as c_int;
    let argv = mem.add(1).cast::<*mut c_char>();
    let envp = argv.add(argc as usize + 1);

    // Do a few more precondition checks on `argc` and `argv`.
    debug_assert!(argc >= 0);
    debug_assert_eq!(*mem, argc as _);
    debug_assert_eq!(*argv.add(argc as usize), std::ptr::null_mut());

    // Explicitly initialize rsix. On non-mustang platforms it uses a
    // .init_array hook to initialize itself automatically, but for mustang,
    // we do it manually so that we can control the initialization order.
    rsix::process::init(envp);

    // Initialize the main thread.
    #[cfg(feature = "threads")]
    crate::threads::initialize_main_thread(mem.cast());

    // For now, print a message, so that we know we're doing something. We'll
    // probably remove this at some point, but for now, things are fragile
    // enough that it's nice to have this confirmation.
    #[cfg(debug_assertions)]
    eprintln!(".｡oO(This process was started by origin! 🎯)");

    // Call the functions registered via `.init_array`.
    #[cfg(not(feature = "initialize-c-runtime"))]
    call_ctors(argc, argv, envp);

    // If enabled, initialize the C runtime. This also handles calling the
    // .init_array functions.
    #[cfg(feature = "initialize-c-runtime")]
    initialize_c_runtime(argc, argv);

    // Call `main`.
    let ret = main(argc, argv, envp);

    // Run functions registered with `at_exit`, and exit with main's return
    // value.
    exit(ret)
}

/// The entrypoint where Rust code is first executed on a new thread.
///
/// # Safety
///
/// It must be valid to call `func`, passing it `arg`, on a new thread, and
/// terminate the thread afterward.
#[cfg(feature = "threads")]
pub(super) unsafe extern "C" fn thread(
    arg: *mut c_void,
    func: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
) -> ! {
    // Do some basic precondition checks, to ensure that our assembly code did
    // what we expect it to do. These are debug-only for now, to keep the
    // release-mode startup code simple to disassemble and inspect, while we're
    // getting started.
    #[cfg(debug_assertions)]
    {
        use crate::current_thread_id;
        use rsix::thread::gettid;

        // Check that the incoming stack pointer is where we expect it to be.
        debug_assert_eq!(builtin_return_address(0), std::ptr::null());
        debug_assert_ne!(builtin_frame_address(0), std::ptr::null());
        #[cfg(not(target_arch = "x86"))]
        debug_assert_eq!(builtin_frame_address(0) as usize & 0xf, 0);
        #[cfg(target_arch = "x86")]
        debug_assert_eq!(builtin_frame_address(0) as usize & 0xf, 8);
        debug_assert_eq!(builtin_frame_address(1), std::ptr::null());
        #[cfg(target_arch = "aarch64")]
        debug_assert_ne!(builtin_sponentry(), std::ptr::null());
        #[cfg(target_arch = "aarch64")]
        debug_assert_eq!(builtin_sponentry() as usize & 0xf, 0);

        // Check that `clone` stored our thread id as we expected.
        debug_assert_eq!(current_thread_id(), gettid());
    }

    // Call the user thread function. In `std`, this is `thread_start`. Ignore
    // the return value for now, as `std` doesn't need it.
    let _result = func(arg);

    exit_thread()
}
