mod dirfd;
mod fcntl;
mod link;
mod lseek;
mod mkdir;
mod open;
mod opendir;
mod readlink;
mod realpath;
mod remove;
mod rename;
mod stat;
mod symlink;
mod sync;
mod truncate;

use rustix::fd::BorrowedFd;
use rustix::ffi::ZStr;
use rustix::fs::AtFlags;

use errno::{set_errno, Errno};
use libc::{c_char, c_int, c_uint, c_void};
#[cfg(not(target_os = "wasi"))]
use memoffset::offset_of;
use std::{convert::TryInto, mem::transmute, ptr::null_mut};

use crate::convert_res;
use crate::fs::opendir::MustangDir;

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
        &BorrowedFd::borrow_raw(dirfd_),
        ZStr::from_ptr(path.cast()),
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

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn readdir64_r(
    dir: *mut c_void,
    entry: *mut libc::dirent64,
    ptr: *mut *mut libc::dirent64,
) -> c_int {
    libc!(libc::readdir64_r(
        dir.cast(),
        checked_cast!(entry),
        checked_cast!(ptr)
    ));

    let mustang_dir = dir.cast::<MustangDir>();
    let dir = &mut (*mustang_dir).dir;
    match dir.read() {
        None => {
            *ptr = null_mut();
            0
        }
        Some(Ok(e)) => {
            let file_type = match e.file_type() {
                rustix::fs::FileType::RegularFile => libc::DT_REG,
                rustix::fs::FileType::Directory => libc::DT_DIR,
                rustix::fs::FileType::Symlink => libc::DT_LNK,
                rustix::fs::FileType::Fifo => libc::DT_FIFO,
                rustix::fs::FileType::Socket => libc::DT_SOCK,
                rustix::fs::FileType::CharacterDevice => libc::DT_CHR,
                rustix::fs::FileType::BlockDevice => libc::DT_BLK,
                rustix::fs::FileType::Unknown => libc::DT_UNKNOWN,
            };
            *entry = libc::dirent64 {
                d_ino: e.ino(),
                d_off: 0, // We don't implement `seekdir` yet anyway.
                d_reclen: (offset_of!(libc::dirent64, d_name) + e.file_name().to_bytes().len() + 1)
                    .try_into()
                    .unwrap(),
                d_type: file_type,
                d_name: [0; 256],
            };
            let len = core::cmp::min(256, e.file_name().to_bytes().len());
            (*entry).d_name[..len].copy_from_slice(transmute(e.file_name().to_bytes()));
            *ptr = entry;
            0
        }
        Some(Err(err)) => err.raw_os_error(),
    }
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn readdir64(dir: *mut c_void) -> *mut libc::dirent64 {
    libc!(libc::readdir64(dir.cast(),));

    let mustang_dir = dir.cast::<MustangDir>();
    let dir = &mut (*mustang_dir).dir;
    match dir.read() {
        None => null_mut(),
        Some(Ok(e)) => {
            let file_type = match e.file_type() {
                rustix::fs::FileType::RegularFile => libc::DT_REG,
                rustix::fs::FileType::Directory => libc::DT_DIR,
                rustix::fs::FileType::Symlink => libc::DT_LNK,
                rustix::fs::FileType::Fifo => libc::DT_FIFO,
                rustix::fs::FileType::Socket => libc::DT_SOCK,
                rustix::fs::FileType::CharacterDevice => libc::DT_CHR,
                rustix::fs::FileType::BlockDevice => libc::DT_BLK,
                rustix::fs::FileType::Unknown => libc::DT_UNKNOWN,
            };
            (*mustang_dir).dirent = libc::dirent64 {
                d_ino: e.ino(),
                d_off: 0, // We don't implement `seekdir` yet anyway.
                d_reclen: (offset_of!(libc::dirent64, d_name) + e.file_name().to_bytes().len() + 1)
                    .try_into()
                    .unwrap(),
                d_type: file_type,
                d_name: [0; 256],
            };
            let len = core::cmp::min(256, e.file_name().to_bytes().len());
            (*mustang_dir).dirent.d_name[..len]
                .copy_from_slice(transmute(e.file_name().to_bytes()));
            &mut (*mustang_dir).dirent
        }
        Some(Err(err)) => {
            set_errno(Errno(err.raw_os_error()));
            null_mut()
        }
    }
}

#[no_mangle]
unsafe extern "C" fn readdir() {
    //libc!(libc::readdir());
    unimplemented!("readdir")
}

#[no_mangle]
unsafe extern "C" fn rewinddir() {
    //libc!(libc::rewinddir());
    unimplemented!("rewinddir")
}

#[no_mangle]
unsafe extern "C" fn scandir() {
    //libc!(libc::scandir());
    unimplemented!("scandir")
}

#[no_mangle]
unsafe extern "C" fn seekdir() {
    //libc!(libc::seekdir());
    unimplemented!("seekdir")
}

#[no_mangle]
unsafe extern "C" fn telldir() {
    //libc!(libc::telldir());
    unimplemented!("telldir")
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
        &BorrowedFd::borrow_raw(fd_in),
        off_in,
        &BorrowedFd::borrow_raw(fd_out),
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
