use rustix::fd::{BorrowedFd, FromRawFd, IntoRawFd, OwnedFd};

use libc::c_int;

use crate::convert_res;

#[no_mangle]
unsafe extern "C" fn dup(fd: c_int) -> c_int {
    libc!(libc::dup(fd));

    match convert_res(rustix::io::dup(&BorrowedFd::borrow_raw(fd))) {
        Some(fd) => OwnedFd::from(fd).into_raw_fd(),
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn dup2(fd: c_int, to: c_int) -> c_int {
    libc!(libc::dup2(fd, to));

    let to = OwnedFd::from_raw_fd(to).into();
    match convert_res(rustix::io::dup2(&BorrowedFd::borrow_raw(fd), &to)) {
        Some(()) => OwnedFd::from(to).into_raw_fd(),
        None => -1,
    }
}