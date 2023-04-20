//! Termios APIs
//!
//! TODO: Clean this up. I think I made the wrong call in rustix, and `Termios`
//! should contain the `termios2` fields. When that's cleaned up, this file can
//! be significantly cleaned up.

use crate::convert_res;
use core::cell::SyncUnsafeCell;
use core::ptr::{copy_nonoverlapping, null_mut};
use libc::{c_char, c_int, size_t};
use rustix::fd::BorrowedFd;
use std::ffi::CString;
use std::vec::Vec;

#[no_mangle]
unsafe extern "C" fn ttyname(fd: c_int) -> *mut c_char {
    libc!(libc::ttyname(fd));

    static STORAGE: SyncUnsafeCell<Option<CString>> = SyncUnsafeCell::new(None);

    let storage = SyncUnsafeCell::get(&STORAGE);
    let name = match convert_res(rustix::termios::ttyname(
        BorrowedFd::borrow_raw(fd),
        Vec::new(),
    )) {
        Some(name) => name,
        None => return null_mut(),
    };
    (*storage) = Some(name);

    (*storage).as_ref().unwrap().as_ptr() as *mut c_char
}

#[no_mangle]
unsafe extern "C" fn ttyname_r(fd: c_int, buf: *mut c_char, buflen: size_t) -> c_int {
    libc!(libc::ttyname_r(fd, buf, buflen));

    let name = match rustix::termios::ttyname(BorrowedFd::borrow_raw(fd), Vec::new()) {
        Ok(name) => name,
        Err(err) => return err.raw_os_error(),
    };

    let len = name.as_bytes().len();
    if len >= buflen {
        return libc::ERANGE;
    }

    copy_nonoverlapping(name.as_ptr(), buf, len);

    0
}
