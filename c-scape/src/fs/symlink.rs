use rustix::fd::BorrowedFd;
use rustix::ffi::ZStr;

use libc::{c_char, c_int};

use crate::convert_res;

#[no_mangle]
unsafe extern "C" fn symlink(target: *const c_char, linkpath: *const c_char) -> c_int {
    libc!(libc::symlink(target, linkpath));

    symlinkat(target, libc::AT_FDCWD, linkpath)
}

#[no_mangle]
unsafe extern "C" fn symlinkat(
    target: *const c_char,
    linkdirfd: c_int,
    linkpath: *const c_char,
) -> c_int {
    libc!(libc::symlinkat(target, linkdirfd, linkpath));

    match convert_res(rustix::fs::symlinkat(
        ZStr::from_ptr(target.cast()),
        BorrowedFd::borrow_raw(linkdirfd),
        ZStr::from_ptr(linkpath.cast()),
    )) {
        Some(()) => 0,
        None => -1,
    }
}
