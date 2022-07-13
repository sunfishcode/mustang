use core::ffi::CStr;
use rustix::fd::BorrowedFd;
use rustix::fs::AtFlags;

use libc::{c_char, c_int};

use crate::convert_res;

#[no_mangle]
unsafe extern "C" fn futimens(fd: c_int, times: *const libc::timespec) -> c_int {
    libc!(libc::futimens(fd, times));

    let timestamps: *const rustix::fs::Timestamps =
        checked_cast!(times as *const [libc::timespec; 2]);

    match convert_res(rustix::fs::futimens(
        BorrowedFd::borrow_raw(fd),
        &*timestamps,
    )) {
        Some(_) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn utimensat(
    fd: c_int,
    path: *const c_char,
    times: *const libc::timespec,
    flag: c_int,
) -> c_int {
    libc!(libc::utimensat(fd, path, times, flag));

    let timestamps: *const rustix::fs::Timestamps =
        checked_cast!(times as *const [libc::timespec; 2]);
    let flags = some_or_ret_einval!(AtFlags::from_bits(flag as _));

    match convert_res(rustix::fs::utimensat(
        BorrowedFd::borrow_raw(fd),
        CStr::from_ptr(path),
        &*timestamps,
        flags,
    )) {
        Some(()) => 0,
        None => -1,
    }
}
