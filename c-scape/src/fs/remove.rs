use core::ffi::CStr;
use rustix::fd::BorrowedFd;
use rustix::fs::AtFlags;

use libc::{c_char, c_int};

use crate::convert_res;

#[no_mangle]
unsafe extern "C" fn rmdir(pathname: *const c_char) -> c_int {
    libc!(libc::rmdir(pathname));

    unlinkat(libc::AT_FDCWD, pathname, libc::AT_REMOVEDIR)
}

#[no_mangle]
unsafe extern "C" fn unlink(pathname: *const c_char) -> c_int {
    libc!(libc::unlink(pathname));

    unlinkat(libc::AT_FDCWD, pathname, 0)
}

#[no_mangle]
unsafe extern "C" fn unlinkat(fd: c_int, pathname: *const c_char, flags: c_int) -> c_int {
    libc!(libc::unlinkat(fd, pathname, flags));

    let fd = BorrowedFd::borrow_raw(fd);
    let flags = AtFlags::from_bits(flags as _).unwrap();
    match convert_res(rustix::fs::unlinkat(
        fd,
        CStr::from_ptr(pathname.cast()),
        flags,
    )) {
        Some(()) => 0,
        None => -1,
    }
}
