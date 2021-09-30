use once_cell::sync::OnceCell;
use std::ffi::c_void;
#[cfg(not(feature = "initialize-c-runtime"))]
use std::os::raw::c_char;
use std::os::raw::c_int;
use std::sync::Mutex;

#[cfg(not(feature = "initialize-c-runtime"))]
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

/// Call all the registered at-exit functions, and exit the process.
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
            (*fini)();
        }
    }

    // Call `exit_immediately` to exit the process.
    exit_immediately(status)
}

/// Exit the process without calling functions registered with `at_exit`.
#[inline]
pub fn exit_immediately(status: c_int) -> ! {
    // For now, print a message, so that we know we're doing something. We'll
    // probably remove this at some point, but for now, things are fragile
    // enough that it's nice to have this confirmation.
    #[cfg(debug_assertions)]
    eprintln!(".｡oO(This process will be exited by c-scape using rsix! 🚪)");

    rsix::process::exit_group(status)
}
