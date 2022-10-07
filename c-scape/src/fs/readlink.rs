use alloc::vec::Vec;
use core::ffi::CStr;
use core::ptr::copy_nonoverlapping;
use rustix::fd::BorrowedFd;

use libc::{c_char, c_int};

use crate::convert_res;

#[no_mangle]
unsafe extern "C" fn readlink(pathname: *const c_char, buf: *mut c_char, bufsiz: usize) -> isize {
    libc!(libc::readlink(pathname, buf, bufsiz));

    readlinkat(libc::AT_FDCWD, pathname, buf, bufsiz)
}

#[no_mangle]
unsafe extern "C" fn readlinkat(
    fd: c_int,
    pathname: *const c_char,
    buf: *mut c_char,
    bufsiz: usize,
) -> isize {
    libc!(libc::readlink(pathname, buf, bufsiz));

    let path = match convert_res(rustix::fs::readlinkat(
        BorrowedFd::borrow_raw(fd),
        CStr::from_ptr(pathname.cast()),
        Vec::new(),
    )) {
        Some(path) => path,
        None => return -1,
    };
    let bytes = path.as_bytes();
    let min = core::cmp::min(bytes.len(), bufsiz);
    copy_nonoverlapping(bytes.as_ptr(), buf.cast(), min);
    min as isize
}
