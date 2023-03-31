use crate::convert_res;
use core::ffi::CStr;
use libc::{c_char, c_int};
use rustix::fd::{BorrowedFd, IntoRawFd};
use rustix::fs::inotify::{self, CreateFlags, WatchFlags};

#[no_mangle]
unsafe extern "C" fn inotify_init() -> c_int {
    libc!(libc::inotify_init());
    inotify_init1(0)
}

#[no_mangle]
unsafe extern "C" fn inotify_init1(flags: c_int) -> c_int {
    libc!(libc::inotify_init1(flags));

    let flags = CreateFlags::from_bits(flags as _).unwrap();
    match convert_res(inotify::inotify_init(flags)) {
        Some(fd) => fd.into_raw_fd(),
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn inotify_add_watch(fd: c_int, pathname: *const c_char, mask: u32) -> c_int {
    libc!(libc::inotify_add_watch(fd, pathname, mask));

    let fd = BorrowedFd::borrow_raw(fd);
    let pathname = CStr::from_ptr(pathname);
    let mask = WatchFlags::from_bits(mask).unwrap();
    match convert_res(inotify::inotify_add_watch(fd, pathname, mask)) {
        Some(wd) => wd,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn inotify_rm_watch(fd: c_int, wd: c_int) -> c_int {
    libc!(libc::inotify_rm_watch(fd, wd));

    let fd = BorrowedFd::borrow_raw(fd);
    match convert_res(inotify::inotify_remove_watch(fd, wd)) {
        Some(()) => 0,
        None => -1,
    }
}
