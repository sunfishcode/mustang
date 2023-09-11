use crate::{set_errno, Errno};
use alloc::borrow::ToOwned;
use alloc::vec::Vec;
use core::cell::SyncUnsafeCell;
use core::ptr::null_mut;
use core::slice;
use libc::{c_char, c_int};
use rustix::ffi::{CStr, CString};

#[no_mangle]
unsafe extern "C" fn getenv(key: *const c_char) -> *mut c_char {
    libc!(libc::getenv(key));

    let key = CStr::from_ptr(key.cast());
    let key_bytes = key.to_bytes();

    _getenv(key_bytes)
}

pub(crate) unsafe fn _getenv(key_bytes: &[u8]) -> *mut c_char {
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
        if key_bytes
            == slice::from_raw_parts(env.cast::<u8>(), c.offset_from(env).try_into().unwrap())
        {
            return c.add(1);
        }
        ptr = ptr.add(1);
    }
    null_mut()
}

/// If we haven't read `environ` yet, or if it changed out from underneath
/// us, read `environ`.
unsafe fn sync_environ(environ_vecs: &mut EnvironVecs) {
    if environ_vecs.ptrs.is_empty() || environ_vecs.ptrs.as_ptr() != environ {
        let mut vecs = EnvironVecs::new();

        let mut ptr = environ;
        loop {
            let env = *ptr;
            if env.is_null() {
                break;
            }
            let owned = CStr::from_ptr(env).to_owned();
            vecs.ptrs.push(owned.as_ptr() as *mut c_char);
            vecs.allocs.push(owned);
            ptr = ptr.add(1);
        }

        vecs.ptrs.push(null_mut());

        *environ_vecs = vecs;
        environ = environ_vecs.ptrs.as_mut_ptr();
    }
}

#[no_mangle]
unsafe extern "C" fn setenv(key: *const c_char, value: *const c_char, overwrite: c_int) -> c_int {
    libc!(libc::setenv(key, value, overwrite));

    if key.is_null() || value.is_null() {
        set_errno(Errno(libc::EINVAL));
        return -1;
    }

    let key = CStr::from_ptr(key);
    let key_bytes = key.to_bytes();

    let value = CStr::from_ptr(value);
    let value_bytes = value.to_bytes();

    if key_bytes.is_empty() || key_bytes.contains(&b'=') {
        set_errno(Errno(libc::EINVAL));
        return -1;
    }

    let environ_vecs = ENVIRON_VECS.get_mut();
    sync_environ(environ_vecs);

    // Construct the new "key=value" string.
    let mut owned = Vec::new();
    owned.extend_from_slice(key_bytes);
    owned.extend_from_slice(b"=");
    owned.extend_from_slice(value_bytes);
    let owned = CString::new(owned).unwrap();

    // Search for the key.
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
        if key_bytes
            == slice::from_raw_parts(env.cast::<u8>(), c.offset_from(env).try_into().unwrap())
        {
            // We found it.
            if overwrite != 0 {
                let index = ptr.offset_from(environ) as usize;
                environ_vecs.ptrs[index] = owned.as_ptr() as *mut c_char;
                environ_vecs.allocs[index] = owned;
            }
            return 0;
        }
        ptr = ptr.add(1);
    }

    // We didn't find the key; append it.
    environ_vecs.ptrs.pop();
    environ_vecs.ptrs.push(owned.as_ptr() as *mut c_char);
    environ_vecs.ptrs.push(null_mut());
    environ_vecs.allocs.push(owned);
    environ = environ_vecs.ptrs.as_mut_ptr();

    0
}

#[no_mangle]
unsafe extern "C" fn unsetenv(key: *const c_char) -> c_int {
    libc!(libc::unsetenv(key));

    if key.is_null() {
        set_errno(Errno(libc::EINVAL));
        return -1;
    }

    let key = CStr::from_ptr(key);
    let key_bytes = key.to_bytes();

    if key_bytes.is_empty() || key_bytes.contains(&b'=') {
        set_errno(Errno(libc::EINVAL));
        return -1;
    }

    let environ_vecs = ENVIRON_VECS.get_mut();
    sync_environ(environ_vecs);

    // Search for the key.
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
        if key_bytes
            == slice::from_raw_parts(env.cast::<u8>(), c.offset_from(env).try_into().unwrap())
        {
            // We found it.
            let index = ptr.offset_from(environ) as usize;
            environ_vecs.ptrs.pop();
            environ_vecs.ptrs.swap_remove(index);
            environ_vecs.ptrs.push(null_mut());
            environ_vecs.allocs.swap_remove(index);
            environ = environ_vecs.ptrs.as_mut_ptr();
            break;
        }
        ptr = ptr.add(1);
    }

    0
}

#[no_mangle]
unsafe extern "C" fn getlogin() -> *mut c_char {
    libc!(libc::getlogin());

    getenv(rustix::cstr!("LOGNAME").as_ptr())
}

/// GLIBC and origin pass argc, argv, and envp to functions in .init_array, as
/// a non-standard extension. Use priority 98 so that we run before any
/// normal user-defined constructor functions and our own functions which
/// depend on `getenv` working.
#[cfg(any(target_env = "gnu", feature = "take-charge"))]
#[link_section = ".init_array.00098"]
#[used]
static INIT_ARRAY: unsafe extern "C" fn(c_int, *mut *mut c_char, *mut *mut c_char) = {
    unsafe extern "C" fn function(_argc: c_int, _argv: *mut *mut c_char, envp: *mut *mut c_char) {
        init_from_envp(envp);
    }
    function
};

#[cfg(not(any(target_env = "gnu", feature = "take-charge")))]
static INIT_ARRAY: Unimplemented = Unimplemented::new();

#[cfg(not(target_os = "wasi"))]
fn init_from_envp(envp: *mut *mut c_char) {
    unsafe {
        environ = envp;
    }
}

#[no_mangle]
pub(crate) static mut environ: *mut *mut c_char = null_mut();

struct EnvironVecs {
    ptrs: Vec<*mut c_char>,
    allocs: Vec<CString>,
}

impl EnvironVecs {
    const fn new() -> Self {
        Self {
            ptrs: Vec::new(),
            allocs: Vec::new(),
        }
    }
}

static mut ENVIRON_VECS: SyncUnsafeCell<EnvironVecs> = SyncUnsafeCell::new(EnvironVecs::new());
