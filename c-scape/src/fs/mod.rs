mod lseek;
mod open;
mod readlink;
mod stat;
mod truncate;

use rustix::fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, OwnedFd, IntoRawFd};
use rustix::ffi::ZStr;
use rustix::fs::{AtFlags, cwd, FdFlags, Mode, OFlags};

use libc::{c_char, c_int, c_void, c_uint};
use errno::{set_errno, Errno};
use std::{convert::TryInto, mem::{transmute, zeroed}, ptr::null_mut, slice};
#[cfg(not(target_os = "wasi"))]
use memoffset::offset_of;

use crate::{convert_res, malloc, memcpy};

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

#[no_mangle]
unsafe extern "C" fn realpath(path: *const c_char, resolved_path: *mut c_char) -> *mut c_char {
    libc!(libc::realpath(path, resolved_path));

    let mut buf = [0; libc::PATH_MAX as usize];
    match realpath_ext::realpath_raw(
        ZStr::from_ptr(path.cast()).to_bytes(),
        &mut buf,
        realpath_ext::RealpathFlags::empty(),
    ) {
        Ok(len) => {
            if resolved_path.is_null() {
                let ptr = malloc(len + 1).cast::<u8>();
                if ptr.is_null() {
                    set_errno(Errno(libc::ENOMEM));
                    return null_mut();
                }
                slice::from_raw_parts_mut(ptr, len).copy_from_slice(&buf[..len]);
                *ptr.add(len) = b'\0';
                ptr.cast::<c_char>()
            } else {
                memcpy(
                    resolved_path.cast::<c_void>(),
                    buf[..len].as_ptr().cast::<c_void>(),
                    len,
                );
                *resolved_path.add(len) = b'\0' as _;
                resolved_path
            }
        }
        Err(err) => {
            set_errno(Errno(err));
            null_mut()
        }
    }
}

#[no_mangle]
unsafe extern "C" fn fcntl(fd: c_int, cmd: c_int, mut args: ...) -> c_int {
    match cmd {
        libc::F_GETFL => {
            libc!(libc::fcntl(fd, libc::F_GETFL));
            let fd = BorrowedFd::borrow_raw(fd);
            match convert_res(rustix::fs::fcntl_getfl(&fd)) {
                Some(flags) => flags.bits() as _,
                None => -1,
            }
        }
        libc::F_SETFD => {
            let flags = args.arg::<c_int>();
            libc!(libc::fcntl(fd, libc::F_SETFD, flags));
            let fd = BorrowedFd::borrow_raw(fd);
            match convert_res(rustix::fs::fcntl_setfd(
                &fd,
                FdFlags::from_bits(flags as _).unwrap(),
            )) {
                Some(()) => 0,
                None => -1,
            }
        }
        #[cfg(not(target_os = "wasi"))]
        libc::F_DUPFD_CLOEXEC => {
            let arg = args.arg::<c_int>();
            libc!(libc::fcntl(fd, libc::F_DUPFD_CLOEXEC, arg));
            let fd = BorrowedFd::borrow_raw(fd);
            match convert_res(rustix::fs::fcntl_dupfd_cloexec(&fd, arg)) {
                Some(fd) => fd.into_raw_fd(),
                None => -1,
            }
        }
        _ => panic!("unrecognized fnctl({})", cmd),
    }
}

#[no_mangle]
unsafe extern "C" fn mkdir(pathname: *const c_char, mode: libc::mode_t) -> c_int {
    libc!(libc::mkdir(pathname, mode));

    let mode = Mode::from_bits((mode & !libc::S_IFMT) as _).unwrap();
    match convert_res(rustix::fs::mkdirat(
        &cwd(),
        ZStr::from_ptr(pathname.cast()),
        mode,
    )) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn fdatasync(fd: c_int) -> c_int {
    libc!(libc::fdatasync(fd));

    match convert_res(rustix::fs::fdatasync(&BorrowedFd::borrow_raw(fd))) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn fsync(fd: c_int) -> c_int {
    libc!(libc::fsync(fd));

    match convert_res(rustix::fs::fdatasync(&BorrowedFd::borrow_raw(fd))) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn rename(old: *const c_char, new: *const c_char) -> c_int {
    libc!(libc::rename(old, new));

    match convert_res(rustix::fs::renameat(
        &cwd(),
        ZStr::from_ptr(old.cast()),
        &cwd(),
        ZStr::from_ptr(new.cast()),
    )) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn rmdir(pathname: *const c_char) -> c_int {
    libc!(libc::rmdir(pathname));

    match convert_res(rustix::fs::unlinkat(
        &cwd(),
        ZStr::from_ptr(pathname.cast()),
        AtFlags::REMOVEDIR,
    )) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn unlinkat(fd: c_int, pathname: *const c_char, flags: c_int) -> c_int {
    libc!(libc::unlinkat(fd, pathname, flags));

    let fd = BorrowedFd::borrow_raw(fd);
    let flags = AtFlags::from_bits(flags as _).unwrap();
    match convert_res(rustix::fs::unlinkat(
        &fd,
        ZStr::from_ptr(pathname.cast()),
        flags,
    )) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn unlink(pathname: *const c_char) -> c_int {
    libc!(libc::unlink(pathname));

    match convert_res(rustix::fs::unlinkat(
        &cwd(),
        ZStr::from_ptr(pathname.cast()),
        AtFlags::empty(),
    )) {
        Some(()) => 0,
        None => -1,
    }
}

struct MustangDir {
    dir: rustix::fs::Dir,
    dirent: libc::dirent64,
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
unsafe extern "C" fn closedir(dir: *mut c_void) -> c_int {
    libc!(libc::closedir(dir.cast()));

    drop(Box::<MustangDir>::from_raw(dir.cast()));
    0
}

#[no_mangle]
unsafe extern "C" fn dirfd(dir: *mut c_void) -> c_int {
    libc!(libc::dirfd(dir.cast()));

    let dir = dir.cast::<MustangDir>();
    (*dir).dir.as_fd().as_raw_fd()
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

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn chmod(pathname: *const c_char, mode: c_uint) -> c_int {
    libc!(libc::chmod(pathname, mode));

    let mode = Mode::from_bits((mode & !libc::S_IFMT) as _).unwrap();
    match convert_res(rustix::fs::chmodat(
        &cwd(),
        ZStr::from_ptr(pathname.cast()),
        mode,
    )) {
        Some(()) => 0,
        None => -1,
    }
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn fchmod(fd: c_int, mode: c_uint) -> c_int {
    libc!(libc::fchmod(fd, mode));

    let mode = Mode::from_bits((mode & !libc::S_IFMT) as _).unwrap();
    match convert_res(rustix::fs::fchmod(&BorrowedFd::borrow_raw(fd), mode)) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn linkat(
    olddirfd: c_int,
    oldpath: *const c_char,
    newdirfd: c_int,
    newpath: *const c_char,
    flags: c_int,
) -> c_int {
    libc!(libc::linkat(olddirfd, oldpath, newdirfd, newpath, flags));

    let flags = AtFlags::from_bits(flags as _).unwrap();
    match convert_res(rustix::fs::linkat(
        &BorrowedFd::borrow_raw(olddirfd),
        ZStr::from_ptr(oldpath.cast()),
        &BorrowedFd::borrow_raw(newdirfd),
        ZStr::from_ptr(newpath.cast()),
        flags,
    )) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn symlink(target: *const c_char, linkpath: *const c_char) -> c_int {
    libc!(libc::symlink(target, linkpath));

    match convert_res(rustix::fs::symlinkat(
        ZStr::from_ptr(target.cast()),
        &cwd(),
        ZStr::from_ptr(linkpath.cast()),
    )) {
        Some(()) => 0,
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