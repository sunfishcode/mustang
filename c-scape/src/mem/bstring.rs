use core::ptr::null_mut;
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

    // It's tempting to use the [memchr crate] to optimize this. However its
    // API requires a Rust slice, which requires that all `len` bytes of the
    // buffer be accessible, while the C `memchr` API guarantees that it stops
    // accessing memory at the first byte that matches.
    //
    // [memchr crate](https://crates.io/crates/memchr)
    for i in 0..len {
        if *s.cast::<u8>().add(i) == c as u8 {
            return s.cast::<u8>().add(i).cast::<c_void>() as *mut c_void;
        }
    }
    null_mut()
}

// Extension: GNU
#[no_mangle]
unsafe extern "C" fn memrchr(s: *const c_void, c: c_int, len: usize) -> *mut c_void {
    libc!(libc::memrchr(s, c, len));

    // As above, it's tempting to use the memchr crate, but the C API here has
    // requirements that we can't meet here.
    for i in 0..len {
        if *s.cast::<u8>().add(len - i - 1) == c as u8 {
            return s.cast::<u8>().add(len - i - 1).cast::<c_void>() as *mut c_void;
        }
    }
    null_mut()
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
