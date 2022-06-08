use core::ffi::CStr;
use rustix::fd::BorrowedFd;
use rustix::fs::{Access, AtFlags};

use libc::{c_char, c_int};

use crate::convert_res;

#[no_mangle]
unsafe extern "C" fn access(pathname: *const c_char, amode: c_int) -> c_int {
    libc!(libc::access(pathname, amode));

    faccessat(libc::AT_FDCWD, pathname, amode, 0)
}

#[no_mangle]
unsafe extern "C" fn faccessat(
    fd: c_int,
    pathname: *const c_char,
    amode: c_int,
    flags: c_int,
) -> c_int {
    libc!(libc::faccessat(fd, pathname, amode, flags));

    match convert_res(rustix::fs::accessat(
        BorrowedFd::borrow_raw(fd),
        CStr::from_ptr(pathname.cast()),
        Access::from_bits(amode as _).unwrap(),
        AtFlags::from_bits(flags as _).unwrap(),
    )) {
        Some(()) => 0,
        None => -1,
    }
}
