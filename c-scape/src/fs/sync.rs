use rustix::fd::BorrowedFd;

use libc::c_int;

use crate::convert_res;

#[no_mangle]
unsafe extern "C" fn fsync(fd: c_int) -> c_int {
    libc!(libc::fsync(fd));

    match convert_res(rustix::fs::fdatasync(&BorrowedFd::borrow_raw(fd))) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn fdatasync(fd: c_int) -> c_int {
    libc!(libc::fdatasync(fd));

    match convert_res(rustix::fs::fdatasync(&BorrowedFd::borrow_raw(fd))) {
        Some(()) => 0,
        None => -1,
    }
}
