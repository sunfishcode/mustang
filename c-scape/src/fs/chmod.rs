use core::ffi::CStr;
use rustix::fd::BorrowedFd;
use rustix::fs::{AtFlags, Mode, CWD};

use libc::{c_char, c_int, mode_t};

use crate::convert_res;

#[no_mangle]
unsafe extern "C" fn chmod(pathname: *const c_char, mode: mode_t) -> c_int {
    libc!(libc::chmod(pathname, mode));

    fchmodat(libc::AT_FDCWD, pathname, mode, 0)
}

#[no_mangle]
unsafe extern "C" fn fchmod(fd: c_int, mode: mode_t) -> c_int {
    libc!(libc::fchmod(fd, mode));

    let mode = Mode::from_bits((mode & !libc::S_IFMT) as _).unwrap();
    match convert_res(rustix::fs::fchmod(BorrowedFd::borrow_raw(fd), mode)) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn fchmodat(
    fd: c_int,
    pathname: *const c_char,
    mode: mode_t,
    flags: c_int,
) -> c_int {
    libc!(libc::fchmodat(fd, pathname, mode, flags));

    if flags != 0 {
        unimplemented!("flags support in fchmodat");
    }

    let mode = Mode::from_bits((mode & !libc::S_IFMT) as _).unwrap();
    match convert_res(rustix::fs::chmodat(
        CWD,
        CStr::from_ptr(pathname.cast()),
        mode,
        AtFlags::empty(),
    )) {
        Some(()) => 0,
        None => -1,
    }
}
