use rustix::fd::BorrowedFd;
use rustix::ffi::ZStr;

use libc::{c_char, c_int};

use crate::convert_res;

#[no_mangle]
unsafe extern "C" fn chdir(path: *const c_char) -> c_int {
    libc!(libc::chdir(path));

    let path = ZStr::from_ptr(path.cast());
    match convert_res(rustix::process::chdir(path)) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn fchdir(fd: c_int) -> c_int {
    libc!(libc::fchdir(fd));

    let fd = BorrowedFd::borrow_raw(fd);
    match convert_res(rustix::process::fchdir(&fd)) {
        Some(()) => 0,
        None => -1,
    }
}
