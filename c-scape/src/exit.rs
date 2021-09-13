use once_cell::sync::Lazy;
use smallvec::SmallVec;
use std::ffi::c_void;
use std::mem::transmute;
use std::os::raw::c_int;
use std::ptr::null_mut;
use std::sync::Mutex;

#[repr(transparent)]
struct SendFunc(unsafe extern "C" fn(*mut c_void));
#[repr(transparent)]
struct SendArg(*mut c_void);

/// Safety: These are only constructed from values passed to `__cxa_atexit`,
/// which is `unsafe`.
unsafe impl Send for SendFunc {}

/// Safety: These are only constructed from values passed to `__cxa_atexit`,
/// which is `unsafe`.
unsafe impl Send for SendArg {}

/// Functions registered to run at exit. [POSIX requires] support for at least
/// 32 atexit functions, so we pre-reserve that much space to ensure we don't
/// fail.
///
/// [POSIX requires]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/atexit.html
static DTORS: Lazy<Mutex<SmallVec<[(SendFunc, SendArg); 32]>>> =
    Lazy::new(|| Mutex::new(SmallVec::new()));

/// Register a function to be called when `exit` is called.
///
/// This function conforms to the [LSB `__cxa_atexit`] ABI.
///
/// [LSB `__cxa_atexit`]: <https://refspecs.linuxbase.org/LSB_5.0.0/LSB-Core-generic/LSB-Core-generic/baselib---cxa-atexit.html>
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn __cxa_atexit(
    func: unsafe extern "C" fn(*mut c_void),
    arg: *mut c_void,
    _dso: *mut c_void,
) -> c_int {
    let mut funcs = DTORS.lock().unwrap();
    let func = SendFunc(func);
    let arg = SendArg(arg);
    match funcs.try_reserve(1) {
        Ok(()) => {
            funcs.push((func, arg));
            0
        }
        Err(_) => -1,
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn __cxa_finalize(_d: *mut c_void) {}

/// C-compatible `atexit`. Consider using `__cxa_atexit` instead.
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn atexit(func: unsafe extern "C" fn()) -> c_int {
    /// Adapter to let `atexit`-style functions be called in the same manner as
    /// `__cxa_atexit` functions.
    unsafe extern "C" fn adapter(func: *mut c_void) {
        transmute::<_, unsafe extern "C" fn()>(func)();
    }

    __cxa_atexit(adapter, func as *mut c_void, null_mut())
}

/// C-compatible `exit`.
///
/// Call all the registered at-exit functions, and exit the process.
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
extern "C" fn exit(status: c_int) -> ! {
    extern "C" {
        static __fini_array_start: c_void;
        static __fini_array_end: c_void;
    }

    // Call all the registered functions, in reverse order. Leave `DTORS`
    // unlocked while making the call so that functions can add more functions
    // to the end of the list.
    while let Some(dtor) = DTORS.lock().unwrap().pop() {
        // Safety: Adding an item to `DTORS` requires `unsafe`, so we assume
        // anything that has been added is safe to call.
        unsafe { dtor.0 .0(dtor.1 .0) };
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

    // Call `_exit` to exit the process.
    _exit(status)
}

/// POSIX-compatible `_exit`.
///
/// Just exit the process.
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
extern "C" fn _exit(status: c_int) -> ! {
    // For now, print a message, so that we know we're doing something. We'll
    // probably remove this at some point, but for now, things are fragile
    // enough that it's nice to have this confirmation.
    #[cfg(debug_assertions)]
    eprintln!(".｡oO(This process will be exited by c-scape using rsix! 🚪)");

    rsix::process::exit_group(status)
}

/// Ensure that this module is linked in.
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
#[cold]
unsafe extern "C" fn __mustang_c_scape__exit() {}
