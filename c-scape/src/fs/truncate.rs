use rustix::fd::BorrowedFd;

use libc::{c_int, off64_t, off_t};

use crate::convert_res;

#[no_mangle]
unsafe extern "C" fn ftruncate(fd: c_int, length: off_t) -> c_int {
    libc!(libc::ftruncate(fd, length));

    ftruncate64(fd, length as off64_t)
}

#[no_mangle]
unsafe extern "C" fn ftruncate64(fd: c_int, length: off64_t) -> c_int {
    libc!(libc::ftruncate64(fd, length));

    match convert_res(rustix::fs::ftruncate(
        BorrowedFd::borrow_raw(fd),
        length as u64,
    )) {
        Some(()) => 0,
        None => -1,
    }
}
