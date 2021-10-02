#[cfg(feature = "threads")]
use crate::threads::initialize_main_thread;
use once_cell::sync::OnceCell;
use std::ffi::c_void;
use std::os::raw::{c_char, c_int};
use std::sync::Mutex;

/// The entrypoint where Rust code is first executed when the program starts.
///
/// # Safety
///
/// `mem` should point to the stack as provided by the operating system.
pub(super) unsafe extern "C" fn entry(mem: *mut usize) -> ! {
    extern "C" {
        fn main(argc: c_int, argv: *mut *mut c_char, envp: *mut *mut c_char) -> c_int;
    }

    // Do some basic precondition checks, to ensure that our assembly code did
    // what we expect it to do. These are debug-only for now, to keep the
    // release-mode startup code simple to disassemble and inspect, while we're
    // getting started.
    #[cfg(debug_assertions)]
    {
        extern "C" {
            #[link_name = "llvm.frameaddress"]
            fn builtin_frame_address(level: i32) -> *const u8;
            #[link_name = "llvm.returnaddress"]
            fn builtin_return_address(level: i32) -> *const u8;
            #[cfg(target_arch = "aarch64")]
            #[link_name = "llvm.sponentry"]
            fn builtin_sponentry() -> *const u8;
        }

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
    // .init_array hook to initialize itself automatically, but for mustang, we
    // do it manually so that we can control the initialization order.
    rsix::process::init(envp);

    // Initialize the main thread.
    #[cfg(feature = "threads")]
    initialize_main_thread(mem.cast());

    // Call the functions registered via `.init_array`.
    call_ctors(argc, argv, envp);

    log::trace!("Calling `main({:?}, {:?}, {:?})`", argc, argv, envp);

    // Call `main`.
    let status = main(argc, argv, envp);

    log::trace!("`main` returned `{:?}`", status);

    // Run functions registered with `at_exit`, and exit with main's return
    // value.
    exit(status)
}

pub(super) unsafe fn call_ctors(argc: c_int, argv: *mut *mut c_char, envp: *mut *mut c_char) {
    extern "C" {
        static __init_array_start: c_void;
        static __init_array_end: c_void;
    }

    // Call the `.init_array` functions.
    type InitFn = fn(c_int, *mut *mut c_char, *mut *mut c_char);
    let mut init = &__init_array_start as *const _ as usize as *const InitFn;
    let init_end = &__init_array_end as *const _ as usize as *const InitFn;
    // Prevent the optimizer from optimizing the `!=` comparison to true;
    // `init` and `init_start` may have the same address.
    asm!("# {}", inout(reg) init);

    while init != init_end {
        log::trace!(
            "Calling `.init_array`-registered function `{:?}({:?}, {:?}, {:?})`",
            *init,
            argc,
            argv,
            envp
        );
        (*init)(argc, argv, envp);
        init = init.add(1);
    }
}

#[repr(transparent)]
struct SendFunc(unsafe extern "C" fn(*mut c_void));
#[repr(transparent)]
struct SendArg(*mut c_void);

/// Safety: Function pointers do not point to data.
unsafe impl Send for SendFunc {}
unsafe impl Send for SendArg {}

/// Functions registerd with `at_exit`.
static DTORS: OnceCell<Mutex<Vec<(SendFunc, SendArg)>>> = OnceCell::new();

/// Register a function to be called when `exit` is called.
///
/// This function conforms to the [LSB `__cxa_atexit`] ABI.
///
/// [LSB `__cxa_atexit`]: <https://refspecs.linuxbase.org/LSB_5.0.0/LSB-Core-generic/LSB-Core-generic/baselib---cxa-atexit.html>
pub fn at_exit(func: unsafe extern "C" fn(*mut c_void), arg: *mut c_void) {
    let dtors = DTORS.get_or_init(|| Mutex::new(Vec::new()));
    let mut funcs = dtors.lock().unwrap();
    funcs.push((SendFunc(func), SendArg(arg)));
}

/// Call all the registered at-exit functions, and exit the program.
pub fn exit(status: c_int) -> ! {
    extern "C" {
        static __fini_array_start: c_void;
        static __fini_array_end: c_void;
    }

    // Call all the registered functions, in reverse order. Leave `DTORS`
    // unlocked while making the call so that functions can add more functions
    // to the end of the list.
    if let Some(dtors) = DTORS.get() {
        while let Some(dtor) = dtors.lock().unwrap().pop() {
            log::trace!(
                "Calling `at_exit`-registered function `{:?}({:?})`",
                dtor.0 .0,
                dtor.1 .0
            );

            // Safety: Adding an item to `DTORS` requires `unsafe`, so we assume
            // anything that has been added is safe to call.
            unsafe { dtor.0 .0(dtor.1 .0) };
        }
    }

    // Call the `.fini_array` functions, in reverse order.
    unsafe {
        type InitFn = fn();
        let mut fini: *const InitFn = &__fini_array_end as *const _ as usize as *const InitFn;
        let fini_start: *const InitFn = &__fini_array_start as *const _ as usize as *const InitFn;
        // Prevent the optimizer from optimizing the `!=` comparison to true;
        // `fini` and `fini_start` may have the same address.
        asm!("# {}", inout(reg) fini);

        while fini != fini_start {
            fini = fini.sub(1);
            log::trace!("Calling `.fini_array`-registered function `{:?}()`", *fini);
            (*fini)();
        }
    }

    // Call `exit_immediately` to exit the process.
    exit_immediately(status)
}

/// Exit the process without calling functions registered with `at_exit`.
#[inline]
pub fn exit_immediately(status: c_int) -> ! {
    log::trace!("Program exiting");

    rsix::process::exit_group(status)
}
