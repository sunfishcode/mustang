use rustix::fd::BorrowedFd;

use core::slice;
use errno::{set_errno, Errno};
use libc::{c_int, c_void, iovec, off64_t, off_t};

use crate::convert_res;

#[no_mangle]
unsafe extern "C" fn write(fd: c_int, ptr: *const c_void, len: usize) -> isize {
    libc!(libc::write(fd, ptr, len));

    match convert_res(rustix::io::write(
        BorrowedFd::borrow_raw(fd),
        slice::from_raw_parts(ptr.cast::<u8>(), len),
    )) {
        Some(nwritten) => nwritten as isize,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn writev(fd: c_int, iov: *const iovec, iovcnt: c_int) -> isize {
    libc!(libc::writev(fd, iov, iovcnt));

    if iovcnt < 0 {
        set_errno(Errno(libc::EINVAL));
        return -1;
    }

    match convert_res(rustix::io::writev(
        BorrowedFd::borrow_raw(fd),
        slice::from_raw_parts(checked_cast!(iov), iovcnt as usize),
    )) {
        Some(nwritten) => nwritten as isize,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn pwrite(fd: c_int, ptr: *const c_void, len: usize, offset: off_t) -> isize {
    libc!(libc::pwrite(fd, ptr, len, offset));
    pwrite64(fd, ptr, len, offset as off64_t)
}

#[no_mangle]
unsafe extern "C" fn pwrite64(fd: c_int, ptr: *const c_void, len: usize, offset: off64_t) -> isize {
    libc!(libc::pwrite64(fd, ptr, len, offset));

    match convert_res(rustix::io::pwrite(
        BorrowedFd::borrow_raw(fd),
        slice::from_raw_parts(ptr.cast::<u8>(), len),
        offset as u64,
    )) {
        Some(nwritten) => nwritten as isize,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn pwritev(fd: c_int, iov: *const iovec, iovcnt: c_int, offset: off_t) -> isize {
    libc!(libc::pwritev(fd, iov, iovcnt, offset));
    pwritev64(fd, iov, iovcnt, offset as off64_t)
}

#[no_mangle]
unsafe extern "C" fn pwritev64(
    fd: c_int,
    iov: *const iovec,
    iovcnt: c_int,
    offset: off64_t,
) -> isize {
    libc!(libc::pwritev64(fd, iov, iovcnt, offset));

    match convert_res(rustix::io::pwritev(
        BorrowedFd::borrow_raw(fd),
        slice::from_raw_parts(checked_cast!(iov), iovcnt as usize),
        offset as u64,
    )) {
        Some(nwritten) => nwritten as isize,
        None => -1,
    }
}
