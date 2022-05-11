use rustix::fd::BorrowedFd;
use rustix::ffi::ZStr;
use rustix::fs::Mode;

use libc::{c_char, c_int};

use crate::convert_res;

#[no_mangle]
unsafe extern "C" fn mkdir(pathname: *const c_char, mode: libc::mode_t) -> c_int {
    libc!(libc::mkdir(pathname, mode));

    mkdirat(libc::AT_FDCWD, pathname, mode)
}

#[no_mangle]
unsafe extern "C" fn mkdirat(fd: c_int, pathname: *const c_char, mode: libc::mode_t) -> c_int {
    libc!(libc::mkdirat(fd, pathname, mode));

    let mode = Mode::from_bits((mode & !libc::S_IFMT) as _).unwrap();
    match convert_res(rustix::fs::mkdirat(
        BorrowedFd::borrow_raw(fd),
        ZStr::from_ptr(pathname.cast()),
        mode,
    )) {
        Some(()) => 0,
        None => -1,
    }
}
