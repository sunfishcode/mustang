use rustix::fd::{BorrowedFd};
use rustix::ffi::ZStr;

use libc::{c_char, c_int};

use crate::convert_res;

#[no_mangle]
unsafe extern "C" fn readlink(pathname: *const c_char, buf: *mut c_char, bufsiz: usize) -> isize {
    libc!(libc::readlink(pathname, buf, bufsiz));

    readlinkat(libc::AT_FDCWD, pathname, buf, bufsiz)
}

#[no_mangle]
unsafe extern "C" fn readlinkat(fd: c_int, pathname: *const c_char, buf: *mut c_char, bufsiz: usize) -> isize {
    libc!(libc::readlink(pathname, buf, bufsiz));

    let path = match convert_res(rustix::fs::readlinkat(
        &BorrowedFd::borrow_raw(fd),
        ZStr::from_ptr(pathname.cast()),
        Vec::new(),
    )) {
        Some(path) => path,
        None => return -1,
    };
    let bytes = path.as_bytes();
    let min = core::cmp::min(bytes.len(), bufsiz);
    std::slice::from_raw_parts_mut(buf.cast(), min).copy_from_slice(bytes);
    min as isize
}