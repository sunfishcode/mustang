use core::{ptr::null_mut, slice};
use libc::{c_int, c_void};

// Obsolescent
#[no_mangle]
unsafe extern "C" fn bcmp(a: *const c_void, b: *const c_void, len: usize) -> c_int {
    // `bcmp` is just an alias for `memcmp`.
    libc!(libc::memcmp(a, b, len));

    compiler_builtins::mem::bcmp(a.cast(), b.cast(), len)
}

#[no_mangle]
unsafe extern "C" fn memchr(s: *const c_void, c: c_int, len: usize) -> *mut c_void {
    libc!(libc::memchr(s, c, len));

    let slice: &[u8] = slice::from_raw_parts(s.cast(), len);
    match memchr::memchr(c as u8, slice) {
        None => null_mut(),
        Some(i) => s.cast::<u8>().add(i) as *mut c_void,
    }
}

// Extension: GNU
#[no_mangle]
unsafe extern "C" fn memrchr(s: *const c_void, c: c_int, len: usize) -> *mut c_void {
    libc!(libc::memrchr(s, c, len));

    let slice: &[u8] = slice::from_raw_parts(s.cast(), len);
    match memchr::memrchr(c as u8, slice) {
        None => null_mut(),
        Some(i) => s.cast::<u8>().add(i) as *mut c_void,
    }
}

#[no_mangle]
unsafe extern "C" fn memcmp(a: *const c_void, b: *const c_void, len: usize) -> c_int {
    libc!(libc::memcmp(a, b, len));

    compiler_builtins::mem::memcmp(a.cast(), b.cast(), len)
}

#[no_mangle]
unsafe extern "C" fn memcpy(dst: *mut c_void, src: *const c_void, len: usize) -> *mut c_void {
    libc!(libc::memcpy(dst, src, len));

    compiler_builtins::mem::memcpy(dst.cast(), src.cast(), len).cast()
}

#[no_mangle]
unsafe extern "C" fn memmove(dst: *mut c_void, src: *const c_void, len: usize) -> *mut c_void {
    libc!(libc::memmove(dst, src, len));

    compiler_builtins::mem::memmove(dst.cast(), src.cast(), len).cast()
}

#[no_mangle]
unsafe extern "C" fn memset(dst: *mut c_void, fill: c_int, len: usize) -> *mut c_void {
    libc!(libc::memset(dst, fill, len));

    compiler_builtins::mem::memset(dst.cast(), fill, len).cast()
}
