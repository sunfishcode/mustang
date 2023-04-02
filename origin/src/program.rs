#[cfg(target_vendor = "mustang")]
use crate::sync::Mutex;
#[cfg(target_vendor = "mustang")]
#[cfg(feature = "threads")]
use crate::threads::initialize_main_thread;
use alloc::boxed::Box;
#[cfg(target_vendor = "mustang")]
use alloc::vec::Vec;
#[cfg(target_vendor = "mustang")]
use core::arch::asm;
use core::ffi::c_void;
#[cfg(not(target_vendor = "mustang"))]
use core::ptr::null_mut;
use linux_raw_sys::ctypes::c_int;

/// The entrypoint where Rust code is first executed when the program starts.
///
/// # Safety
///
/// `mem` should point to the stack as provided by the operating system.
#[cfg(target_vendor = "mustang")]
pub(super) unsafe extern "C" fn entry(mem: *mut usize) -> ! {
    extern "C" {
        fn main(argc: c_int, argv: *mut *mut u8, envp: *mut *mut u8) -> c_int;
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
        debug_assert_ne!(mem, core::ptr::null_mut());
        debug_assert_eq!(mem.addr() & 0xf, 0);
        debug_assert!(builtin_frame_address(0).addr() <= mem.addr());

        // Check that the incoming stack pointer is where we expect it to be.
        debug_assert_eq!(builtin_return_address(0), core::ptr::null());
        debug_assert_ne!(builtin_frame_address(0), core::ptr::null());
        #[cfg(not(any(target_arch = "arm", target_arch = "x86")))]
        debug_assert_eq!(builtin_frame_address(0).addr() & 0xf, 0);
        #[cfg(target_arch = "arm")]
        debug_assert_eq!(builtin_frame_address(0).addr() & 0x7, 0);
        #[cfg(target_arch = "x86")]
        debug_assert_eq!(builtin_frame_address(0).addr() & 0xf, 8);
        debug_assert_eq!(builtin_frame_address(1), core::ptr::null());
        #[cfg(target_arch = "aarch64")]
        debug_assert_ne!(builtin_sponentry(), core::ptr::null());
        #[cfg(target_arch = "aarch64")]
        debug_assert_eq!(builtin_sponentry().addr() & 0xf, 0);
    }

    // Compute `argc`, `argv`, and `envp`.
    let argc = *mem as c_int;
    let argv = mem.add(1).cast::<*mut u8>();
    let envp = argv.add(argc as usize + 1);

    // Do a few more precondition checks on `argc` and `argv`.
    debug_assert!(argc >= 0);
    debug_assert_eq!(*mem, argc as _);
    debug_assert_eq!(*argv.add(argc as usize), core::ptr::null_mut());

    // Explicitly initialize `rustix`. On non-mustang platforms, `rustix` uses
    // a .init_array hook to initialize itself automatically, but for mustang,
    // we do it manually so that we can control the initialization order.
    rustix::param::init(envp);

    // Initialize the main thread.
    #[cfg(feature = "threads")]
    initialize_main_thread(mem.cast());

    // Call the functions registered via `.init_array`.
    call_ctors(argc, argv, envp);

    #[cfg(feature = "log")]
    log::trace!("Calling `main({:?}, {:?}, {:?})`", argc, argv, envp);

    // Call `main`.
    let status = main(argc, argv, envp);

    #[cfg(feature = "log")]
    log::trace!("`main` returned `{:?}`", status);

    // Run functions registered with `at_exit`, and exit with main's return
    // value.
    exit(status)
}

/// Call the constructors in the `.init_array` section.
#[cfg(target_vendor = "mustang")]
unsafe fn call_ctors(argc: c_int, argv: *mut *mut u8, envp: *mut *mut u8) {
    extern "C" {
        static __init_array_start: c_void;
        static __init_array_end: c_void;
    }

    // Call the `.init_array` functions. As GLIBC, does, pass argc, argv,
    // and envp as extra arguments. In addition to GLIBC ABI compatibility,
    // c-scape relies on this.
    type InitFn = fn(c_int, *mut *mut u8, *mut *mut u8);
    let mut init = &__init_array_start as *const _ as *const InitFn;
    let init_end = &__init_array_end as *const _ as *const InitFn;
    // Prevent the optimizer from optimizing the `!=` comparison to true;
    // `init` and `init_start` may have the same address.
    asm!("# {}", inout(reg) init);

    while init != init_end {
        #[cfg(feature = "log")]
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

/// Functions registered with [`at_exit`].
#[cfg(target_vendor = "mustang")]
static DTORS: Mutex<Vec<Box<dyn FnOnce() + Send>>> = Mutex::new(Vec::new());

/// Register a function to be called when [`exit`] is called.
///
/// # Safety
///
/// This arranges for `func` to be called, and passed `obj`, when the program
/// exits.
pub fn at_exit(func: Box<dyn FnOnce() + Send>) {
    #[cfg(target_vendor = "mustang")]
    {
        let mut funcs = DTORS.lock();
        funcs.push(func);
    }

    #[cfg(not(target_vendor = "mustang"))]
    {
        extern "C" {
            // <https://refspecs.linuxbase.org/LSB_3.1.0/LSB-generic/LSB-generic/baselib---cxa-atexit.html>
            fn __cxa_atexit(
                func: unsafe extern "C" fn(*mut c_void),
                arg: *mut c_void,
                _dso: *mut c_void,
            ) -> c_int;
        }

        // The function to pass to `__cxa_atexit`.
        unsafe extern "C" fn at_exit_func(arg: *mut c_void) {
            Box::from_raw(arg as *mut Box<dyn FnOnce() + Send>)();
        }

        let at_exit_arg = Box::into_raw(Box::new(func)).cast::<c_void>();
        let r = unsafe { __cxa_atexit(at_exit_func, at_exit_arg, null_mut()) };
        assert_eq!(r, 0);
    }
}

/// Call all the functions registered with [`at_exit`] or with the
/// `.fini_array` section, and exit the program.
pub fn exit(status: c_int) -> ! {
    // Call functions registered with `at_thread_exit`.
    #[cfg(target_vendor = "mustang")]
    #[cfg(feature = "threads")]
    crate::threads::call_thread_dtors(crate::current_thread());

    // Call all the registered functions, in reverse order. Leave `DTORS`
    // unlocked while making the call so that functions can add more functions
    // to the end of the list.
    #[cfg(target_vendor = "mustang")]
    loop {
        let mut dtors = DTORS.lock();
        if let Some(dtor) = dtors.pop() {
            #[cfg(feature = "log")]
            log::trace!("Calling `at_exit`-registered function");

            dtor();
        } else {
            break;
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
        let mut fini: *const InitFn = &__fini_array_end as *const _ as *const InitFn;
        let fini_start: *const InitFn = &__fini_array_start as *const _ as *const InitFn;
        // Prevent the optimizer from optimizing the `!=` comparison to true;
        // `fini` and `fini_start` may have the same address.
        asm!("# {}", inout(reg) fini);

        while fini != fini_start {
            fini = fini.sub(1);

            #[cfg(feature = "log")]
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

/// Exit the program without calling functions registered with [`at_exit`] or
/// with the `.fini_array` section.
#[inline]
pub fn exit_immediately(status: c_int) -> ! {
    #[cfg(feature = "log")]
    log::trace!("Program exiting");

    #[cfg(target_vendor = "mustang")]
    {
        // Call `rustix` to exit the program.
        rustix::runtime::exit_group(status)
    }

    #[cfg(not(target_vendor = "mustang"))]
    unsafe {
        // Call `libc` to exit the program.
        libc::_exit(status)
    }
}
