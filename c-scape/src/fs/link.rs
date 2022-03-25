use rustix::fd::{BorrowedFd};
use rustix::ffi::ZStr;
use rustix::fs::{AtFlags};

use libc::{c_char, c_int};

use crate::convert_res;

#[no_mangle]
unsafe extern "C" fn link(
    oldpath: *const c_char,
    newpath: *const c_char,
) -> c_int {
    libc!(libc::link(oldpath, newpath));

    linkat(libc::AT_FDCWD, oldpath, libc::AT_FDCWD, newpath, 0)
}

#[no_mangle]
unsafe extern "C" fn linkat(
    olddirfd: c_int,
    oldpath: *const c_char,
    newdirfd: c_int,
    newpath: *const c_char,
    flags: c_int,
) -> c_int {
    libc!(libc::linkat(olddirfd, oldpath, newdirfd, newpath, flags));

    let flags = AtFlags::from_bits(flags as _).unwrap();
    match convert_res(rustix::fs::linkat(
        &BorrowedFd::borrow_raw(olddirfd),
        ZStr::from_ptr(oldpath.cast()),
        &BorroewdFd::borrow_raw(newdirfd),
        ZStr::from_ptr(newpath.cast()),
        flags,
    )) {
        Some(()) => 0,
        None => -1,
    }
}
