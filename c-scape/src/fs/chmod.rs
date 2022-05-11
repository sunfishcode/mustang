use rustix::fd::BorrowedFd;
use rustix::ffi::ZStr;
use rustix::fs::{cwd, Mode};

use libc::{c_char, c_int, c_uint};

use crate::convert_res;

#[no_mangle]
unsafe extern "C" fn chmod(pathname: *const c_char, mode: c_uint) -> c_int {
    libc!(libc::chmod(pathname, mode));

    let mode = Mode::from_bits((mode & !libc::S_IFMT) as _).unwrap();
    match convert_res(rustix::fs::chmodat(
        cwd(),
        ZStr::from_ptr(pathname.cast()),
        mode,
    )) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn fchmod(fd: c_int, mode: c_uint) -> c_int {
    libc!(libc::fchmod(fd, mode));

    let mode = Mode::from_bits((mode & !libc::S_IFMT) as _).unwrap();
    match convert_res(rustix::fs::fchmod(BorrowedFd::borrow_raw(fd), mode)) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn fchmodat() {
    // libc!(libc::fchmodat());
    unimplemented!("fchmodat")
}
