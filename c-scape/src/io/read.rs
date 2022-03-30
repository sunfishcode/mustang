use rustix::fd::BorrowedFd;
use rustix::io::IoSliceMut;

use errno::{set_errno, Errno};
use libc::{c_int, c_void, iovec, off64_t, off_t};
use std::slice;

use crate::convert_res;

#[no_mangle]
unsafe extern "C" fn read(fd: c_int, ptr: *mut c_void, len: usize) -> isize {
    libc!(libc::read(fd, ptr, len));

    match convert_res(rustix::io::read(
        &BorrowedFd::borrow_raw(fd),
        slice::from_raw_parts_mut(ptr.cast::<u8>(), len),
    )) {
        Some(nread) => nread as isize,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn readv(fd: c_int, iov: *const iovec, iovcnt: c_int) -> isize {
    libc!(libc::readv(fd, iov, iovcnt));

    if iovcnt < 0 {
        set_errno(Errno(libc::EINVAL));
        return -1;
    }

    let iov: *const IoSliceMut = checked_cast!(iov);

    // Note that rustix's `readv` takes a `&mut`, however it doesn't
    // mutate the `IoSliceMut` instances themselves, so it's safe to
    // cast away the `const` here.
    match convert_res(rustix::io::readv(
        &BorrowedFd::borrow_raw(fd),
        slice::from_raw_parts_mut(iov as *mut _, iovcnt as usize),
    )) {
        Some(nread) => nread as isize,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn pread(fd: c_int, ptr: *mut c_void, len: usize, offset: off_t) -> isize {
    libc!(libc::pread(fd, ptr, len, offset));
    pread64(fd, ptr, len, offset as off64_t)
}

#[no_mangle]
unsafe extern "C" fn pread64(fd: c_int, ptr: *mut c_void, len: usize, offset: off64_t) -> isize {
    libc!(libc::pread64(fd, ptr, len, offset));

    match convert_res(rustix::io::pread(
        &BorrowedFd::borrow_raw(fd),
        slice::from_raw_parts_mut(ptr.cast::<u8>(), len),
        offset as u64,
    )) {
        Some(nread) => nread as isize,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn preadv(fd: c_int, iov: *const iovec, iovcnt: c_int, offset: off_t) -> isize {
    libc!(libc::preadv(fd, iov, iovcnt, offset));
    preadv64(fd, iov, iovcnt, offset as off64_t)
}

#[no_mangle]
unsafe extern "C" fn preadv64(
    fd: c_int,
    iov: *const iovec,
    iovcnt: c_int,
    offset: off64_t,
) -> isize {
    libc!(libc::preadv64(fd, iov, iovcnt, offset));

    let iov: *const IoSliceMut = checked_cast!(iov);

    // Note that rustix's `readv` takes a `&mut`, however it doesn't
    // mutate the `IoSliceMut` instances themselves, so it's safe to
    // cast away the `const` here.
    match convert_res(rustix::io::preadv(
        &BorrowedFd::borrow_raw(fd),
        slice::from_raw_parts_mut(iov as *mut _, iovcnt as usize),
        offset as u64,
    )) {
        Some(nwritten) => nwritten as isize,
        None => -1,
    }
}
