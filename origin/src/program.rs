#[cfg(target_vendor = "mustang")]
use crate::mutex::Mutex;
#[cfg(target_vendor = "mustang")]
#[cfg(feature = "threads")]
use crate::threads::{initialize_main_thread, set_current_thread_id};
use alloc::boxed::Box;
#[cfg(target_vendor = "mustang")]
use alloc::vec::Vec;
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
        debug_assert_eq!(mem as usize & 0xf, 0);
        debug_assert!((builtin_frame_address(0) as usize) <= mem as usize);

        // Check that the incoming stack pointer is where we expect it to be.
        debug_assert_eq!(builtin_return_address(0), core::ptr::null());
        debug_assert_ne!(builtin_frame_address(0), core::ptr::null());
        #[cfg(not(any(target_arch = "arm", target_arch = "x86")))]
        debug_assert_eq!(builtin_frame_address(0) as usize & 0xf, 0);
        #[cfg(target_arch = "arm")]
        debug_assert_eq!(builtin_frame_address(0) as usize & 0x7, 0);
        #[cfg(target_arch = "x86")]
        debug_assert_eq!(builtin_frame_address(0) as usize & 0xf, 8);
        debug_assert_eq!(builtin_frame_address(1), core::ptr::null());
        #[cfg(target_arch = "aarch64")]
        debug_assert_ne!(builtin_sponentry(), core::ptr::null());
        #[cfg(target_arch = "aarch64")]
        debug_assert_eq!(builtin_sponentry() as usize & 0xf, 0);
    }

    // Compute `argc`, `argv`, and `envp`.
    let argc = *mem as c_int;
    let argv = mem.add(1).cast::<*mut u8>();
    let envp = argv.add(argc as usize + 1);

    // Do a few more precondition checks on `argc` and `argv`.
    debug_assert!(argc >= 0);
    debug_assert_eq!(*mem, argc as _);
    debug_assert_eq!(*argv.add(argc as usize), core::ptr::null_mut());

    // Explicitly initialize `rustix`. On non-mustang platforms it uses a
    // .init_array hook to initialize itself automatically, but for mustang, we
    // do it manually so that we can control the initialization order.
    rustix::process::init(envp);

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
pub(super) unsafe fn call_ctors(argc: c_int, argv: *mut *mut u8, envp: *mut *mut u8) {
    extern "C" {
        static __init_array_start: c_void;
        static __init_array_end: c_void;
    }

    // Call the `.init_array` functions.
    type InitFn = fn(c_int, *mut *mut u8, *mut *mut u8);
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

/// Functions registered with `at_exit`.
#[cfg(target_vendor = "mustang")]
static DTORS: Mutex<Vec<Box<dyn FnOnce() + Send>>> = Mutex::new(Vec::new());

/// Register a function to be called when `exit` is called.
///
/// # Safety
///
/// This arranges for `func` to be called, and passed `obj`, when the program
/// exits.
pub fn at_exit(func: Box<dyn FnOnce() + Send>) {
    #[cfg(target_vendor = "mustang")]
    {
        DTORS.lock(|funcs| funcs.push(func));
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
        unsafe extern "C" fn at_exit_func(arg: *mut c_void) {
            Box::from_raw(arg as *mut Box<dyn FnOnce() + Send>)();
        }
        let at_exit_arg = Box::into_raw(Box::new(func)).cast::<c_void>();
        let r = unsafe { __cxa_atexit(at_exit_func, at_exit_arg, null_mut()) };
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
    while let Some(dtor) = DTORS.lock(|dtors| dtors.pop()) {
        log::trace!("Calling `at_exit`-registered function",);

        dtor();
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
    #[cfg(target_vendor = "mustang")]
    log::trace!("Program exiting");

    rustix::runtime::exit_group(status)
}

#[cfg(target_vendor = "mustang")]
#[derive(Default)]
struct RegisteredForkFuncs {
    pub(crate) prepare: Vec<unsafe extern "C" fn()>,
    pub(crate) parent: Vec<unsafe extern "C" fn()>,
    pub(crate) child: Vec<unsafe extern "C" fn()>,
}

#[cfg(target_vendor = "mustang")]
impl RegisteredForkFuncs {
    pub(crate) const fn new() -> Self {
        Self {
            prepare: Vec::new(),
            parent: Vec::new(),
            child: Vec::new(),
        }
    }
}

/// Functions registered with `at_fork`.
#[cfg(target_vendor = "mustang")]
static FORK_FUNCS: Mutex<RegisteredForkFuncs> = Mutex::new(RegisteredForkFuncs::new());

/// Register functions to be called when `fork` is called.
///
/// The handlers for each phase are called in the following order:
/// the prepare handlers are called in reverse order of registration;
/// the parent and child handlers are called in the order of registration.
#[cfg(not(target_os = "wasi"))]
pub fn at_fork(
    prepare_func: Option<unsafe extern "C" fn()>,
    parent_func: Option<unsafe extern "C" fn()>,
    child_func: Option<unsafe extern "C" fn()>,
) {
    #[cfg(target_vendor = "mustang")]
    {
        FORK_FUNCS.lock(|funcs| {
            if let Some(func) = prepare_func {
                funcs.prepare.push(func);
            }
            if let Some(func) = parent_func {
                funcs.parent.push(func);
            }
            if let Some(func) = child_func {
                funcs.child.push(func);
            }
        });
    }

    #[cfg(not(target_vendor = "mustang"))]
    {
        extern "C" {
            fn __register_atfork(
                prepare: Option<unsafe extern "C" fn()>,
                parent: Option<unsafe extern "C" fn()>,
                child: Option<unsafe extern "C" fn()>,
                dso_handle: *mut c_void,
            ) -> c_int;
        }
        let r = unsafe { __register_atfork(prepare_func, parent_func, child_func, null_mut()) };
        assert_eq!(r, 0);
    }
}

/// Creates a new process by duplicating the calling process.
///
/// # Safety
///
/// After a fork in a multithreaded process returns in the child,
/// data structures such as mutexes might be in an unusable state.
/// code should use `at_fork` to either protect them,
/// from being used by other threads during the fork,
/// or reinitialize them in child process.
#[cfg(not(target_os = "wasi"))]
pub unsafe fn fork() -> rustix::io::Result<Option<rustix::process::Pid>> {
    #[cfg(target_vendor = "mustang")]
    {
        FORK_FUNCS.lock(|funcs| {
            funcs.prepare.iter().rev().for_each(|func| func());
            // Protect the allocator lock while the fork is in progress, to
            // avoid deadlocks in the child process from allocations.
            match crate::allocator::INNER_ALLOC.lock(|_| rustix::runtime::fork())? {
                rustix::process::Pid::NONE => {
                    #[cfg(feature = "threads")]
                    set_current_thread_id(rustix::thread::gettid());
                    funcs.child.iter().for_each(|func| func());
                    Ok(None)
                }
                pid => {
                    funcs.parent.iter().for_each(|func| func());
                    Ok(Some(pid))
                }
            }
        })
    }

    #[cfg(not(target_vendor = "mustang"))]
    {
        extern "C" {
            fn fork() -> c_int;
        }
        match fork() {
            -1 => {
                let raw = *libc::__errno_location();
                Err(rustix::io::Error::from_raw_os_error(raw))
            }
            0 => Ok(None),
            pid => Ok(Some(rustix::process::Pid::from_raw(pid as _))),
        }
    }
}
