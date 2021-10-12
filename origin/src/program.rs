#[cfg(target_vendor = "mustang")]
#[cfg(feature = "threads")]
use crate::threads::initialize_main_thread;
#[cfg(target_vendor = "mustang")]
use once_cell::sync::OnceCell;
use std::ffi::c_void;
#[cfg(target_vendor = "mustang")]
use std::os::raw::c_char;
use std::os::raw::c_int;
#[cfg(not(target_vendor = "mustang"))]
use std::ptr::null_mut;
#[cfg(target_vendor = "mustang")]
use std::sync::Mutex;

/// The entrypoint where Rust code is first executed when the program starts.
///
/// # Safety
///
/// `mem` should point to the stack as provided by the operating system.
#[cfg(target_vendor = "mustang")]
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
        #[cfg(not(any(target_arch = "arm", target_arch = "x86")))]
        debug_assert_eq!(builtin_frame_address(0) as usize & 0xf, 0);
        #[cfg(target_arch = "arm")]
        debug_assert_eq!(builtin_frame_address(0) as usize & 0x7, 0);
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

#[cfg(target_vendor = "mustang")]
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

/// Functions registered with `at_exit`.
#[cfg(target_vendor = "mustang")]
static DTORS: OnceCell<Mutex<Vec<(SendFunc, SendArg)>>> = OnceCell::new();

/// Register a function to be called when `exit` is called.
///
/// # Safety
///
/// This arranges for `func` to be called, and passed `obj`, when the program
/// exits.
pub unsafe fn at_exit(func: unsafe extern "C" fn(*mut c_void), arg: *mut c_void) {
    #[cfg(target_vendor = "mustang")]
    {
        let dtors = DTORS.get_or_init(|| Mutex::new(Vec::new()));
        let mut funcs = dtors.lock().unwrap();
        funcs.push((SendFunc(func), SendArg(arg)));
    }

    #[cfg(not(target_vendor = "mustang"))]
    {
        extern "C" {
            fn __cxa_atexit(
                func: unsafe extern "C" fn(*mut c_void),
                arg: *mut c_void,
                _dso: *mut c_void,
            ) -> c_int;
        }
        let r = __cxa_atexit(func, arg, null_mut());
        assert_eq!(r, 0);
    }
}

/// Call all the functions registered with `at_exit` and `.fini_array`, and
/// exit the program.
pub fn exit(status: c_int) -> ! {
    // Call all the registered functions, in reverse order. Leave `DTORS`
    // unlocked while making the call so that functions can add more functions
    // to the end of the list.
    #[cfg(target_vendor = "mustang")]
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
    #[cfg(target_vendor = "mustang")]
    unsafe {
        extern "C" {
            static __fini_array_start: c_void;
            static __fini_array_end: c_void;
        }

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

    #[cfg(target_vendor = "mustang")]
    {
        // Call `exit_immediately` to exit the program.
        exit_immediately(status)
    }

    #[cfg(not(target_vendor = "mustang"))]
    unsafe {
        // Call `libc` to run *its* dtors, and exit the program.
        libc::exit(status)
    }
}

/// Exit the program without calling functions registered with `at_exit` or
/// `.fini_array`.
#[inline]
pub fn exit_immediately(status: c_int) -> ! {
    log::trace!("Program exiting");

    rsix::process::exit_group(status)
}
