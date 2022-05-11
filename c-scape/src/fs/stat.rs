use rustix::fd::BorrowedFd;
use rustix::ffi::ZStr;
use rustix::fs::AtFlags;

use core::convert::TryInto;
use errno::{set_errno, Errno};
use libc::{c_char, c_int, time_t};

use crate::convert_res;

fn rustix_stat_to_libc_stat(
    rustix_stat: rustix::fs::Stat,
) -> Result<libc::stat, std::num::TryFromIntError> {
    // SAFETY: libc structs can be zero-initalized freely
    let mut stat: libc::stat = unsafe { std::mem::zeroed() };
    stat.st_dev = rustix_stat.st_dev.try_into()?;
    stat.st_ino = rustix_stat.st_ino.try_into()?;
    stat.st_nlink = rustix_stat.st_nlink.try_into()?;
    stat.st_mode = rustix_stat.st_mode.try_into()?;
    stat.st_uid = rustix_stat.st_uid.try_into()?;
    stat.st_gid = rustix_stat.st_gid.try_into()?;
    stat.st_rdev = rustix_stat.st_rdev.try_into()?;
    stat.st_size = rustix_stat.st_size.try_into()?;
    stat.st_blksize = rustix_stat.st_blksize.try_into()?;
    stat.st_blocks = rustix_stat.st_blocks.try_into()?;
    stat.st_atime = rustix_stat.st_atime.try_into()?;
    stat.st_atime_nsec = rustix_stat.st_atime_nsec.try_into()?;
    stat.st_mtime = rustix_stat.st_mtime.try_into()?;
    stat.st_mtime_nsec = rustix_stat.st_mtime_nsec.try_into()?;
    stat.st_ctime = rustix_stat.st_ctime as time_t;
    stat.st_ctime_nsec = rustix_stat.st_ctime_nsec.try_into()?;
    Ok(stat)
}

#[no_mangle]
unsafe extern "C" fn stat(pathname: *const c_char, stat_: *mut libc::stat) -> c_int {
    libc!(libc::stat(pathname, stat_));

    fstatat(libc::AT_FDCWD, pathname, stat_, 0)
}

#[no_mangle]
unsafe extern "C" fn stat64(pathname: *const c_char, stat_: *mut libc::stat64) -> c_int {
    libc!(libc::stat64(pathname, stat_));

    fstatat64(libc::AT_FDCWD, pathname, stat_, 0)
}

#[no_mangle]
unsafe extern "C" fn lstat(pathname: *const c_char, stat_: *mut libc::stat) -> c_int {
    libc!(libc::lstat(pathname, stat_));

    fstatat(libc::AT_FDCWD, pathname, stat_, libc::AT_SYMLINK_NOFOLLOW)
}

#[no_mangle]
unsafe extern "C" fn lstat64(pathname: *const c_char, stat_: *mut libc::stat64) -> c_int {
    libc!(libc::lstat64(pathname, stat_));

    fstatat64(libc::AT_FDCWD, pathname, stat_, libc::AT_SYMLINK_NOFOLLOW)
}

#[no_mangle]
unsafe extern "C" fn fstatat(
    fd: c_int,
    pathname: *const c_char,
    stat_: *mut libc::stat,
    flags: c_int,
) -> c_int {
    libc!(libc::fstatat(fd, pathname, stat_, flags));

    let flags = AtFlags::from_bits(flags as _).unwrap();
    let rustix_stat = match convert_res(rustix::fs::statat(
        BorrowedFd::borrow_raw(fd),
        ZStr::from_ptr(pathname.cast()),
        flags,
    )) {
        Some(r) => r,
        None => return -1,
    };

    match rustix_stat_to_libc_stat(rustix_stat) {
        Ok(r) => {
            *stat_ = r;
            0
        }
        Err(_) => {
            set_errno(Errno(libc::EOVERFLOW));
            -1
        }
    }
}

#[no_mangle]
unsafe extern "C" fn fstatat64(
    fd: c_int,
    pathname: *const c_char,
    stat_: *mut libc::stat64,
    flags: c_int,
) -> c_int {
    libc!(libc::fstatat64(fd, pathname, stat_, flags));

    let stat_: *mut rustix::fs::Stat = checked_cast!(stat_);

    let flags = AtFlags::from_bits(flags as _).unwrap();
    match convert_res(rustix::fs::statat(
        BorrowedFd::borrow_raw(fd),
        ZStr::from_ptr(pathname.cast()),
        flags,
    )) {
        Some(r) => {
            *stat_ = r;
            0
        }
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn fstat(fd: c_int, stat_: *mut libc::stat) -> c_int {
    libc!(libc::fstat(fd, stat_));

    let rustix_stat = match convert_res(rustix::fs::fstat(BorrowedFd::borrow_raw(fd))) {
        Some(r) => r,
        None => return -1,
    };

    match rustix_stat_to_libc_stat(rustix_stat) {
        Ok(r) => {
            *stat_ = r;
            0
        }
        Err(_) => {
            set_errno(Errno(libc::EOVERFLOW));
            -1
        }
    }
}

#[no_mangle]
unsafe extern "C" fn fstat64(fd: c_int, stat_: *mut libc::stat64) -> c_int {
    libc!(libc::fstat64(fd, checked_cast!(stat_)));

    let stat_: *mut rustix::fs::Stat = checked_cast!(stat_);

    match convert_res(rustix::fs::fstat(BorrowedFd::borrow_raw(fd))) {
        Some(r) => {
            *stat_ = r;
            0
        }
        None => -1,
    }
}
