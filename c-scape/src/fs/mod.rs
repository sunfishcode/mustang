mod access;
#[cfg(not(target_os = "wasi"))]
mod chmod;
mod dir;
mod fcntl;
mod link;
mod lseek;
mod mkdir;
mod open;
mod readlink;
mod realpath;
mod remove;
mod rename;
mod stat;
mod symlink;
mod sync;
mod truncate;

use core::ffi::CStr;
use rustix::fd::BorrowedFd;
use rustix::fs::AtFlags;

use errno::{set_errno, Errno};
use libc::{c_char, c_int, c_uint};

use crate::convert_res;

#[cfg(any(target_os = "android", target_os = "linux"))]
#[no_mangle]
unsafe extern "C" fn statx(
    dirfd_: c_int,
    path: *const c_char,
    flags: c_int,
    mask: c_uint,
    stat_: *mut rustix::fs::Statx,
) -> c_int {
    libc!(libc::statx(dirfd_, path, flags, mask, checked_cast!(stat_)));

    if path.is_null() || stat_.is_null() {
        set_errno(Errno(libc::EFAULT));
        return -1;
    }

    let flags = AtFlags::from_bits(flags as _).unwrap();
    let mask = rustix::fs::StatxFlags::from_bits(mask).unwrap();
    match convert_res(rustix::fs::statx(
        BorrowedFd::borrow_raw(dirfd_),
        CStr::from_ptr(path.cast()),
        flags,
        mask,
    )) {
        Some(r) => {
            *stat_ = r;
            0
        }
        None => -1,
    }
}

#[cfg(any(target_os = "android", target_os = "linux"))]
#[no_mangle]
unsafe extern "C" fn copy_file_range(
    fd_in: c_int,
    off_in: *mut i64,
    fd_out: c_int,
    off_out: *mut i64,
    len: usize,
    flags: c_uint,
) -> isize {
    libc!(libc::copy_file_range(
        fd_in, off_in, fd_out, off_out, len, flags
    ));

    if fd_in == -1 || fd_out == -1 {
        set_errno(Errno(libc::EBADF));
        return -1;
    }
    assert_eq!(flags, 0);
    let off_in = if off_in.is_null() {
        None
    } else {
        Some(&mut *off_in.cast::<u64>())
    };
    let off_out = if off_out.is_null() {
        None
    } else {
        Some(&mut *off_out.cast::<u64>())
    };
    match convert_res(rustix::fs::copy_file_range(
        BorrowedFd::borrow_raw(fd_in),
        off_in,
        BorrowedFd::borrow_raw(fd_out),
        off_out,
        len as u64,
    )) {
        Some(n) => n as _,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn chown() {
    //libc!(libc::chown());
    unimplemented!("chown")
}

#[no_mangle]
unsafe extern "C" fn lchown() {
    //libc!(libc::lchown());
    unimplemented!("lchown")
}

#[no_mangle]
unsafe extern "C" fn fchown() {
    //libc!(libc::fchown());
    unimplemented!("fchown")
}
