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

#[no_mangle]
unsafe extern "C" fn setxattr(
    path: *const c_char,
    name: *const c_char,
    value: *const c_void,
    size: size_t,
    flags: c_int,
) -> c_int {
    libc!(libc::setxattr(path, name, value, size, flags));
    set_errno(Errno(libc::ENOTSUP));
    -1
}

#[no_mangle]
unsafe extern "C" fn lsetxattr(
    path: *const c_char,
    name: *const c_char,
    value: *const c_void,
    size: size_t,
    flags: c_int,
) -> c_int {
    libc!(libc::lsetxattr(path, name, value, size, flags));
    set_errno(Errno(libc::ENOTSUP));
    -1
}

#[no_mangle]
unsafe extern "C" fn fsetxattr(
    fd: c_int,
    name: *const c_char,
    value: *const c_void,
    size: size_t,
    flags: c_int,
) -> c_int {
    libc!(libc::fsetxattr(fd, name, value, size, flags));
    set_errno(Errno(libc::ENOTSUP));
    -1
}

#[no_mangle]
unsafe extern "C" fn listxattr(path: *const c_char, list: *mut c_char, size: size_t) -> ssize_t {
    libc!(libc::listxattr(path, list, size));
    set_errno(Errno(libc::ENOTSUP));
    -1
}

#[no_mangle]
unsafe extern "C" fn llistxattr(path: *const c_char, list: *mut c_char, size: size_t) -> ssize_t {
    libc!(libc::llistxattr(path, list, size));
    set_errno(Errno(libc::ENOTSUP));
    -1
}

#[no_mangle]
unsafe extern "C" fn flistxattr(fd: c_int, list: *mut c_char, size: size_t) -> ssize_t {
    libc!(libc::flistxattr(fd, list, size));
    set_errno(Errno(libc::ENOTSUP));
    -1
}
