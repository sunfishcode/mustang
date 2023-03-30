//! Extended attributes.
//!
//! These are not yet implemented in rustix. For now, use stubs so that
//! programs that use them can be linked.

use crate::{set_errno, Errno};
use libc::{c_char, c_int, c_void, size_t, ssize_t};

#[no_mangle]
unsafe extern "C" fn getxattr(
    path: *const c_char,
    name: *const c_char,
    value: *mut c_void,
    size: size_t,
) -> ssize_t {
    libc!(libc::getxattr(path, name, value, size));
    set_errno(Errno(libc::ENOTSUP));
    -1
}

#[no_mangle]
unsafe extern "C" fn lgetxattr(
    path: *const c_char,
    name: *const c_char,
    value: *mut c_void,
    size: size_t,
) -> ssize_t {
    libc!(libc::lgetxattr(path, name, value, size));
    set_errno(Errno(libc::ENOTSUP));
    -1
}

#[no_mangle]
unsafe extern "C" fn fgetxattr(
    fd: c_int,
    name: *const c_char,
    value: *mut c_void,
    size: size_t,
) -> ssize_t {
    libc!(libc::fgetxattr(fd, name, value, size));
    set_errno(Errno(libc::ENOTSUP));
    -1
}
