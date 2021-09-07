use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::ptr::null_mut;
use std::slice;

#[no_mangle]
pub unsafe extern "C" fn getenv(key: *const c_char) -> *mut c_char {
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

/// GLIBC passes argc, argv, and envp to functions in .init_array, as a
/// non-standard extension. Use priority 99 so that we run before any
/// normal user-defined constructor functions.
#[cfg(target_env = "gnu")]
#[used]
#[link_section = ".init_array.00099"]
static INIT_ARRAY: unsafe extern "C" fn(c_int, *mut *mut c_char, *mut *mut c_char) = {
    unsafe extern "C" fn function(_argc: c_int, _argv: *mut *mut c_char, envp: *mut *mut c_char) {
        init_from_envp(envp);
    }
    function
};

/// For musl etc., assume that `__environ` is available and points to the
/// original environment from the kernel, so we can find the auxv array in
/// memory after it. Use priority 99 so that we run before any normal
/// user-defined constructor functions.
#[cfg(not(target_env = "gnu"))]
#[used]
#[link_section = ".init_array.00099"]
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
    // For now, print a message, so that we know we're doing something. We'll
    // probably remove this at some point, but for now, things are fragile
    // enough that it's nice to have this confirmation.
    #[cfg(debug_assertions)]
    eprintln!(".｡oO(Environment variables initialized by c-scape! 🌱)");

    unsafe { environ = envp };
}

#[cfg(target_env = "gnu")]
#[no_mangle]
#[used]
pub static mut environ: *mut *mut c_char = null_mut();
