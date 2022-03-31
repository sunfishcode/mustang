use rustix::fd::{FromRawFd, IntoRawFd, OwnedFd};
use rustix::ffi::ZStr;
use rustix::fs::{cwd, Mode, OFlags};

use core::{mem::zeroed, ptr::null_mut};
use libc::{c_char, c_int, c_void};

use crate::convert_res;

pub(super) struct MustangDir {
    pub(super) dir: rustix::fs::Dir,
    pub(super) dirent: libc::dirent64,
}

#[no_mangle]
unsafe extern "C" fn opendir(pathname: *const c_char) -> *mut c_void {
    libc!(libc::opendir(pathname).cast());

    match convert_res(rustix::fs::openat(
        &cwd(),
        ZStr::from_ptr(pathname.cast()),
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
        Mode::empty(),
    )) {
        Some(fd) => fdopendir(fd.into_raw_fd()),
        None => null_mut(),
    }
}

#[no_mangle]
unsafe extern "C" fn fdopendir(fd: c_int) -> *mut c_void {
    libc!(libc::fdopendir(fd).cast());

    match convert_res(rustix::fs::Dir::from(OwnedFd::from_raw_fd(fd))) {
        Some(dir) => Box::into_raw(Box::new(MustangDir {
            dir,
            dirent: zeroed(),
        }))
        .cast(),
        None => null_mut(),
    }
}

#[no_mangle]
unsafe extern "C" fn closedir(dir: *mut c_void) -> c_int {
    libc!(libc::closedir(dir.cast()));

    drop(Box::<MustangDir>::from_raw(dir.cast()));
    0
}
