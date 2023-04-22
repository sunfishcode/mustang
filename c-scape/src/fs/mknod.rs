use core::ffi::CStr;
use rustix::fd::BorrowedFd;
use rustix::fs::{FileType, Mode};

use libc::{c_char, c_int};

use crate::convert_res;

#[no_mangle]
unsafe extern "C" fn mkfifo(pathname: *const c_char, mode: libc::mode_t) -> c_int {
    libc!(libc::mkfifo(pathname, mode));

    mkfifoat(libc::AT_FDCWD, pathname, mode)
}

#[no_mangle]
unsafe extern "C" fn mkfifoat(fd: c_int, pathname: *const c_char, mode: libc::mode_t) -> c_int {
    libc!(libc::mkfifoat(fd, pathname, mode));

    mknodat(fd, pathname, mode | libc::S_IFIFO, 0)
}

#[no_mangle]
unsafe extern "C" fn mknod(pathname: *const c_char, mode: libc::mode_t, dev: libc::dev_t) -> c_int {
    libc!(libc::mknod(pathname, mode, dev));

    mknodat(libc::AT_FDCWD, pathname, mode, dev)
}

#[no_mangle]
unsafe extern "C" fn mknodat(
    dirfd: c_int,
    pathname: *const c_char,
    mode: libc::mode_t,
    dev: libc::dev_t,
) -> c_int {
    libc!(libc::mknodat(dirfd, pathname, mode, dev));

    let filetype = match mode & libc::S_IFMT {
        libc::S_IFREG => FileType::RegularFile,
        libc::S_IFDIR => FileType::Directory,
        libc::S_IFLNK => FileType::Symlink,
        libc::S_IFIFO => FileType::Fifo,
        libc::S_IFSOCK => FileType::Socket,
        libc::S_IFCHR => FileType::CharacterDevice,
        libc::S_IFBLK => FileType::BlockDevice,
        _ => FileType::Unknown,
    };
    let mode = Mode::from_bits((mode & !libc::S_IFMT) as _).unwrap();

    match convert_res(rustix::fs::mknodat(
        BorrowedFd::borrow_raw(dirfd),
        CStr::from_ptr(pathname.cast()),
        filetype,
        mode,
        dev,
    )) {
        Some(()) => 0,
        None => -1,
    }
}
