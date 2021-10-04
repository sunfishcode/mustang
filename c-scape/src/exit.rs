use std::ffi::c_void;
use std::mem::transmute;
use std::os::raw::c_int;

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
    origin::at_exit(func, arg);
    0
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
    libc!(atexit(status));

    /// Adapter to let `atexit`-style functions be called in the same manner as
    /// `__cxa_atexit` functions.
    unsafe extern "C" fn adapter(func: *mut c_void) {
        transmute::<_, unsafe extern "C" fn()>(func)();
    }

    origin::at_exit(adapter, func as *mut c_void);
    0
}

/// C-compatible `exit`.
///
/// Call all the registered at-exit functions, and exit the program.
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
extern "C" fn exit(status: c_int) -> ! {
    libc!(exit(status));
    origin::exit(status)
}

/// POSIX-compatible `_exit`.
///
/// Just exit the process.
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
extern "C" fn _exit(status: c_int) -> ! {
    libc!(_exit(status));
    origin::exit_immediately(status)
}

/// Ensure that this module is linked in.
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
#[cold]
unsafe extern "C" fn __mustang_c_scape__exit() {}
