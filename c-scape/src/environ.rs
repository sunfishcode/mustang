use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::ptr::null_mut;
use std::slice;

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn getenv(key: *const c_char) -> *mut c_char {
    libc!(getenv(key));
    let key = CStr::from_ptr(key);
    let mut ptr = environ;
    loop {
        let env = *ptr;
        if env.is_null() {
            break;
        }
        let mut c = env;
        while *c != (b'=' as c_char) {
            c = c.add(1);
        }
        if key.to_bytes() == slice::from_raw_parts(env.cast::<u8>(), (c as usize) - (env as usize))
        {
            return c.add(1);
        }
        ptr = ptr.add(1);
    }
    null_mut()
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn setenv() {
    //libc!(setenv());
    unimplemented!("setenv")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn unsetenv() {
    //libc!(unsetenv());
    unimplemented!("unsetenv")
}

/// GLIBC passes argc, argv, and envp to functions in .init_array, as a
/// non-standard extension. Use priority 98 so that we run before any
/// normal user-defined constructor functions and our own functions which
/// depend on `getenv` working.
#[cfg(target_env = "gnu")]
#[link_section = ".init_array.00098"]
#[used]
static INIT_ARRAY: unsafe extern "C" fn(c_int, *mut *mut c_char, *mut *mut c_char) = {
    unsafe extern "C" fn function(_argc: c_int, _argv: *mut *mut c_char, envp: *mut *mut c_char) {
        init_from_envp(envp);
    }
    function
};

/// For musl etc., assume that `__environ` is available and points to the
/// original environment from the kernel, so we can find the auxv array in
/// memory after it. Use priority 98 so that we run before any normal
/// user-defined constructor functions and our own functions which
/// depend on `getenv` working.
///
/// <https://refspecs.linuxbase.org/LSB_5.0.0/LSB-Core-generic/LSB-Core-generic/baselib---environ.html>
#[cfg(not(target_env = "gnu"))]
#[link_section = ".init_array.00098"]
#[used]
static INIT_ARRAY: unsafe extern "C" fn() = {
    unsafe extern "C" fn function() {
        extern "C" {
            static __environ: *mut *mut c_char;
        }

        init_from_envp(__environ)
    }
    function
};

fn init_from_envp(envp: *mut *mut c_char) {
    unsafe {
        environ = envp;
    }
}

#[link_section = ".data.__mustang"]
#[no_mangle]
#[used]
static mut environ: *mut *mut c_char = null_mut();

/// Ensure that this module is linked in.
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
#[cold]
unsafe extern "C" fn __mustang_c_scape__environ() {}
