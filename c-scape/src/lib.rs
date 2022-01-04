#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]
#![no_builtins] // don't let LLVM optimize our `memcpy` into a `memcpy` call
#![feature(thread_local)] // for `__errno_location`
#![feature(c_variadic)] // for `ioctl` etc.
#![feature(const_fn_fn_ptr_basics)] // for creating an empty `Vec::new` of function pointers in a const fn
#![feature(rustc_private)] // for compiler-builtins
#![feature(untagged_unions)] // for `PthreadMutexT`
#![feature(atomic_mut_ptr)] // for `RawMutex`

extern crate alloc;
extern crate compiler_builtins;

#[macro_use]
mod use_libc;

#[cfg(not(target_os = "wasi"))]
mod at_fork;
mod error_str;
// Unwinding isn't supported on 32-bit arm yet.
#[cfg(target_arch = "arm")]
mod unwind;

// Selected libc-compatible interfaces.
//
// The goal here isn't necessarily to build a complete libc; it's primarily
// to provide things that `std` and possibly popular crates are currently
// using.
//
// This effectively undoes the work that `rustix` does: it calls `rustix` and
// translates it back into a C-like ABI. Ideally, Rust code should just call
// the `rustix` APIs directly, which are safer, more ergonomic, and skip this
// whole layer.

use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::format;
#[cfg(feature = "sync-resolve")]
use alloc::string::ToString;
use alloc::vec::Vec;
use core::convert::TryInto;
use core::ffi::c_void;
#[cfg(not(target_arch = "riscv64"))]
use core::mem::align_of;
#[cfg(not(target_os = "wasi"))]
use core::mem::{size_of, transmute, zeroed};
use core::ptr::{self, null, null_mut};
use core::slice;
#[cfg(feature = "threads")]
use core::sync::atomic::Ordering::SeqCst;
#[cfg(feature = "threads")]
use core::sync::atomic::{AtomicBool, AtomicU32};
use errno::{set_errno, Errno};
use error_str::error_str;
use libc::{c_char, c_int, c_long, c_uint, c_ulong};
#[cfg(not(target_os = "wasi"))]
use memoffset::offset_of;
#[cfg(feature = "threads")]
use origin::Thread;
#[cfg(feature = "threads")]
use parking_lot::lock_api::{RawMutex as _, RawRwLock};
#[cfg(feature = "threads")]
use parking_lot::RawMutex;
use rustix::fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, IntoRawFd};
use rustix::ffi::ZStr;
use rustix::fs::{cwd, openat, AtFlags, FdFlags, Mode, OFlags};
#[cfg(any(target_os = "android", target_os = "linux"))]
use rustix::io::EventfdFlags;
use rustix::io::OwnedFd;
#[cfg(not(target_os = "wasi"))]
use rustix::io::{MapFlags, MprotectFlags, MremapFlags, PipeFlags, ProtFlags};
#[cfg(feature = "net")]
use rustix::net::{
    AcceptFlags, AddressFamily, Ipv4Addr, Ipv6Addr, Protocol, RecvFlags, SendFlags, Shutdown,
    SocketAddrAny, SocketAddrStorage, SocketFlags, SocketType,
};
#[cfg(not(target_os = "wasi"))]
use rustix::process::{Pid, WaitOptions};
#[cfg(feature = "sync-resolve")]
use {
    rustix::net::{IpAddr, SocketAddrV4, SocketAddrV6},
    sync_resolve::resolve_host,
};

// errno

#[no_mangle]
unsafe extern "C" fn __errno_location() -> *mut c_int {
    libc!(libc::__errno_location());

    #[cfg_attr(feature = "threads", thread_local)]
    static mut ERRNO: i32 = 0;
    &mut ERRNO
}

#[no_mangle]
unsafe extern "C" fn __xpg_strerror_r(errnum: c_int, buf: *mut c_char, buflen: usize) -> c_int {
    libc!(libc::strerror_r(errnum, buf, buflen));

    let message = match error_str(rustix::io::Error::from_raw_os_error(errnum)) {
        Some(s) => s.to_owned(),
        None => format!("Unknown error {}", errnum),
    };
    let min = core::cmp::min(buflen, message.len());
    let out = slice::from_raw_parts_mut(buf.cast::<u8>(), min);
    out.copy_from_slice(message.as_bytes());
    out[out.len() - 1] = b'\0';
    0
}

// fs

#[no_mangle]
unsafe extern "C" fn open64(pathname: *const c_char, flags: c_int, mode: c_int) -> c_int {
    libc!(libc::open64(pathname, flags, mode));

    let flags = OFlags::from_bits(flags as _).unwrap();
    let mode = Mode::from_bits(mode as _).unwrap();
    match convert_res(openat(&cwd(), ZStr::from_ptr(pathname.cast()), flags, mode)) {
        Some(fd) => fd.into_raw_fd(),
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn open() {
    //libc!(libc::open())
    unimplemented!("open")
}

#[no_mangle]
unsafe extern "C" fn readlink(pathname: *const c_char, buf: *mut c_char, bufsiz: usize) -> isize {
    libc!(libc::readlink(pathname, buf, bufsiz));

    let path = match convert_res(rustix::fs::readlinkat(
        &cwd(),
        ZStr::from_ptr(pathname.cast()),
        Vec::new(),
    )) {
        Some(path) => path,
        None => return -1,
    };
    let bytes = path.as_bytes();
    let min = core::cmp::min(bytes.len(), bufsiz);
    slice::from_raw_parts_mut(buf.cast::<_>(), min).copy_from_slice(bytes);
    min as isize
}

#[no_mangle]
unsafe extern "C" fn stat() -> c_int {
    //libc!(libc::stat())
    unimplemented!("stat")
}

#[no_mangle]
unsafe extern "C" fn stat64(pathname: *const c_char, stat_: *mut rustix::fs::Stat) -> c_int {
    libc!(libc::stat64(pathname, same_ptr_mut(stat_)));

    match convert_res(rustix::fs::statat(
        &cwd(),
        ZStr::from_ptr(pathname.cast()),
        AtFlags::empty(),
    )) {
        Some(r) => {
            *stat_ = r;
            0
        }
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn fstat(_fd: c_int, _stat: *mut rustix::fs::Stat) -> c_int {
    libc!(libc::fstat(_fd, same_ptr_mut(_stat)));
    unimplemented!("fstat")
}

#[no_mangle]
unsafe extern "C" fn fstat64(fd: c_int, stat_: *mut rustix::fs::Stat) -> c_int {
    libc!(libc::fstat64(fd, same_ptr_mut(stat_)));

    match convert_res(rustix::fs::fstat(&BorrowedFd::borrow_raw_fd(fd))) {
        Some(r) => {
            *stat_ = r;
            0
        }
        None => -1,
    }
}

#[cfg(any(target_os = "android", target_os = "linux"))]
#[no_mangle]
unsafe extern "C" fn statx(
    dirfd_: c_int,
    path: *const c_char,
    flags: c_int,
    mask: c_uint,
    stat_: *mut rustix::fs::Statx,
) -> c_int {
    libc!(libc::statx(dirfd_, path, flags, mask, same_ptr_mut(stat_)));

    if path.is_null() || stat_.is_null() {
        set_errno(Errno(libc::EFAULT));
        return -1;
    }

    let flags = AtFlags::from_bits(flags as _).unwrap();
    let mask = rustix::fs::StatxFlags::from_bits(mask).unwrap();
    match convert_res(rustix::fs::statx(
        &BorrowedFd::borrow_raw_fd(dirfd_),
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
            let fd = BorrowedFd::borrow_raw_fd(fd);
            match convert_res(rustix::fs::fcntl_getfl(&fd)) {
                Some(flags) => flags.bits() as _,
                None => -1,
            }
        }
        libc::F_SETFD => {
            let flags = args.arg::<c_int>();
            libc!(libc::fcntl(fd, libc::F_SETFD, flags));
            let fd = BorrowedFd::borrow_raw_fd(fd);
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
            let fd = BorrowedFd::borrow_raw_fd(fd);
            match convert_res(rustix::fs::fcntl_dupfd_cloexec(&fd, arg)) {
                Some(fd) => fd.into_raw_fd(),
                None => -1,
            }
        }
        _ => panic!("unrecognized fnctl({})", cmd),
    }
}

#[no_mangle]
unsafe extern "C" fn mkdir(pathname: *const c_char, mode: c_uint) -> c_int {
    libc!(libc::mkdir(pathname, mode));

    let mode = Mode::from_bits(mode as _).unwrap();
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

    match convert_res(rustix::fs::fdatasync(&BorrowedFd::borrow_raw_fd(fd))) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn fstatat64(
    fd: c_int,
    pathname: *const c_char,
    stat_: *mut rustix::fs::Stat,
    flags: c_int,
) -> c_int {
    libc!(libc::fstatat64(fd, pathname, same_ptr_mut(stat_), flags));

    let flags = AtFlags::from_bits(flags as _).unwrap();
    match convert_res(rustix::fs::statat(
        &BorrowedFd::borrow_raw_fd(fd),
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
unsafe extern "C" fn fsync(fd: c_int) -> c_int {
    libc!(libc::fsync(fd));

    match convert_res(rustix::fs::fdatasync(&BorrowedFd::borrow_raw_fd(fd))) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn ftruncate64(fd: c_int, length: i64) -> c_int {
    libc!(libc::ftruncate64(fd, length));

    match convert_res(rustix::fs::ftruncate(
        &BorrowedFd::borrow_raw_fd(fd),
        length as u64,
    )) {
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

#[no_mangle]
unsafe extern "C" fn lseek64(fd: c_int, offset: i64, whence: c_int) -> i64 {
    libc!(libc::lseek64(fd, offset, whence));

    let seek_from = match whence {
        libc::SEEK_SET => rustix::io::SeekFrom::Start(offset as u64),
        libc::SEEK_CUR => rustix::io::SeekFrom::Current(offset),
        libc::SEEK_END => rustix::io::SeekFrom::End(offset),
        _ => panic!("unrecognized whence({})", whence),
    };
    match convert_res(rustix::fs::seek(&BorrowedFd::borrow_raw_fd(fd), seek_from)) {
        Some(offset) => offset as i64,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn lstat64(pathname: *const c_char, stat_: *mut rustix::fs::Stat) -> c_int {
    libc!(libc::lstat64(pathname, same_ptr_mut(stat_)));

    match convert_res(rustix::fs::statat(
        &cwd(),
        ZStr::from_ptr(pathname.cast()),
        AtFlags::SYMLINK_NOFOLLOW,
    )) {
        Some(r) => {
            *stat_ = r;
            0
        }
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn opendir(pathname: *const c_char) -> *mut c_void {
    libc!(libc::opendir(pathname).cast::<_>());

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
    libc!(libc::fdopendir(fd).cast::<_>());

    match convert_res(rustix::fs::Dir::from(OwnedFd::from_raw_fd(fd))) {
        Some(dir) => Box::into_raw(Box::new(dir)).cast::<_>(),
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
        dir.cast::<_>(),
        same_ptr_mut(entry),
        same_ptr_mut(ptr)
    ));

    let dir = dir.cast::<rustix::fs::Dir>();
    match (*dir).read() {
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

#[no_mangle]
unsafe extern "C" fn closedir(dir: *mut c_void) -> c_int {
    libc!(libc::closedir(dir.cast::<_>()));

    drop(Box::<rustix::fs::Dir>::from_raw(dir.cast()));
    0
}

#[no_mangle]
unsafe extern "C" fn dirfd(dir: *mut c_void) -> c_int {
    libc!(libc::dirfd(dir.cast::<_>()));

    let dir = dir.cast::<rustix::fs::Dir>();
    (*dir).as_fd().as_raw_fd()
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
        &BorrowedFd::borrow_raw_fd(fd_in),
        off_in,
        &BorrowedFd::borrow_raw_fd(fd_out),
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

    let mode = Mode::from_bits(mode as _).unwrap();
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

    let mode = Mode::from_bits(mode as _).unwrap();
    match convert_res(rustix::fs::fchmod(&BorrowedFd::borrow_raw_fd(fd), mode)) {
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
        &BorrowedFd::borrow_raw_fd(olddirfd),
        ZStr::from_ptr(oldpath.cast()),
        &BorrowedFd::borrow_raw_fd(newdirfd),
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

// net

#[cfg(feature = "net")]
#[no_mangle]
unsafe extern "C" fn accept(
    fd: c_int,
    addr: *mut SocketAddrStorage,
    len: *mut libc::socklen_t,
) -> c_int {
    libc!(libc::accept(fd, same_ptr_mut(addr), len));

    match convert_res(rustix::net::acceptfrom(&BorrowedFd::borrow_raw_fd(fd))) {
        Some((accepted_fd, from)) => {
            let encoded_len = from.write(addr);
            *len = encoded_len.try_into().unwrap();
            accepted_fd.into_raw_fd()
        }
        None => -1,
    }
}

#[cfg(feature = "net")]
#[no_mangle]
unsafe extern "C" fn accept4(
    fd: c_int,
    addr: *mut SocketAddrStorage,
    len: *mut libc::socklen_t,
    flags: c_int,
) -> c_int {
    libc!(libc::accept4(fd, same_ptr_mut(addr), len, flags));

    let flags = AcceptFlags::from_bits(flags as _).unwrap();
    match convert_res(rustix::net::acceptfrom_with(
        &BorrowedFd::borrow_raw_fd(fd),
        flags,
    )) {
        Some((accepted_fd, from)) => {
            let encoded_len = from.write(addr);
            *len = encoded_len.try_into().unwrap();
            accepted_fd.into_raw_fd()
        }
        None => -1,
    }
}

#[cfg(feature = "net")]
#[no_mangle]
unsafe extern "C" fn bind(
    sockfd: c_int,
    addr: *const SocketAddrStorage,
    len: libc::socklen_t,
) -> c_int {
    libc!(libc::bind(sockfd, same_ptr(addr), len));

    let addr = match convert_res(SocketAddrAny::read(addr, len.try_into().unwrap())) {
        Some(addr) => addr,
        None => return -1,
    };
    match convert_res(match addr {
        SocketAddrAny::V4(v4) => rustix::net::bind_v4(&BorrowedFd::borrow_raw_fd(sockfd), &v4),
        SocketAddrAny::V6(v6) => rustix::net::bind_v6(&BorrowedFd::borrow_raw_fd(sockfd), &v6),
        SocketAddrAny::Unix(unix) => {
            rustix::net::bind_unix(&BorrowedFd::borrow_raw_fd(sockfd), &unix)
        }
        _ => panic!("unrecognized SocketAddrAny kind"),
    }) {
        Some(()) => 0,
        None => -1,
    }
}

#[cfg(feature = "net")]
#[no_mangle]
unsafe extern "C" fn connect(
    sockfd: c_int,
    addr: *const SocketAddrStorage,
    len: libc::socklen_t,
) -> c_int {
    libc!(libc::connect(sockfd, same_ptr(addr), len));

    let addr = match convert_res(SocketAddrAny::read(addr, len.try_into().unwrap())) {
        Some(addr) => addr,
        None => return -1,
    };
    match convert_res(match addr {
        SocketAddrAny::V4(v4) => rustix::net::connect_v4(&BorrowedFd::borrow_raw_fd(sockfd), &v4),
        SocketAddrAny::V6(v6) => rustix::net::connect_v6(&BorrowedFd::borrow_raw_fd(sockfd), &v6),
        SocketAddrAny::Unix(unix) => {
            rustix::net::connect_unix(&BorrowedFd::borrow_raw_fd(sockfd), &unix)
        }
        _ => panic!("unrecognized SocketAddrAny kind"),
    }) {
        Some(()) => 0,
        None => -1,
    }
}

#[cfg(feature = "net")]
#[no_mangle]
unsafe extern "C" fn getpeername(
    fd: c_int,
    addr: *mut SocketAddrStorage,
    len: *mut libc::socklen_t,
) -> c_int {
    libc!(libc::getpeername(fd, same_ptr_mut(addr), len));

    match convert_res(rustix::net::getpeername(&BorrowedFd::borrow_raw_fd(fd))) {
        Some(from) => {
            let encoded_len = from.write(addr);
            *len = encoded_len.try_into().unwrap();
            0
        }
        None => -1,
    }
}

#[cfg(feature = "net")]
#[no_mangle]
unsafe extern "C" fn getsockname(
    fd: c_int,
    addr: *mut SocketAddrStorage,
    len: *mut libc::socklen_t,
) -> c_int {
    libc!(libc::getsockname(fd, same_ptr_mut(addr), len));

    match convert_res(rustix::net::getsockname(&BorrowedFd::borrow_raw_fd(fd))) {
        Some(from) => {
            let encoded_len = from.write(addr);
            *len = encoded_len.try_into().unwrap();
            0
        }
        None => -1,
    }
}

#[cfg(feature = "net")]
#[no_mangle]
unsafe extern "C" fn getsockopt(
    fd: c_int,
    level: c_int,
    optname: c_int,
    optval: *mut c_void,
    optlen: *mut libc::socklen_t,
) -> c_int {
    use core::time::Duration;
    use rustix::net::sockopt::{self, Timeout};

    unsafe fn write_bool(
        value: rustix::io::Result<bool>,
        optval: *mut c_void,
        optlen: *mut libc::socklen_t,
    ) -> rustix::io::Result<()> {
        Ok(write(value? as c_uint, optval.cast::<c_uint>(), optlen))
    }

    unsafe fn write_u32(
        value: rustix::io::Result<u32>,
        optval: *mut c_void,
        optlen: *mut libc::socklen_t,
    ) -> rustix::io::Result<()> {
        Ok(write(value?, optval.cast::<u32>(), optlen))
    }

    unsafe fn write_linger(
        linger: rustix::io::Result<Option<Duration>>,
        optval: *mut c_void,
        optlen: *mut libc::socklen_t,
    ) -> rustix::io::Result<()> {
        let linger = linger?;
        let linger = libc::linger {
            l_onoff: linger.is_some() as c_int,
            l_linger: linger.unwrap_or_default().as_secs() as c_int,
        };
        Ok(write(linger, optval.cast::<libc::linger>(), optlen))
    }

    unsafe fn write_timeval(
        value: rustix::io::Result<Option<Duration>>,
        optval: *mut c_void,
        optlen: *mut libc::socklen_t,
    ) -> rustix::io::Result<()> {
        let timeval = match value? {
            None => libc::timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            Some(duration) => libc::timeval {
                tv_sec: duration
                    .as_secs()
                    .try_into()
                    .map_err(|_| rustix::io::Error::OVERFLOW)?,
                tv_usec: duration.subsec_micros() as _,
            },
        };
        Ok(write(timeval, optval.cast::<libc::timeval>(), optlen))
    }

    unsafe fn write<T>(value: T, optval: *mut T, optlen: *mut libc::socklen_t) {
        *optlen = size_of::<T>().try_into().unwrap();
        optval.write(value)
    }

    libc!(libc::getsockopt(fd, level, optname, optval, optlen));

    let fd = &BorrowedFd::borrow_raw_fd(fd);
    let result = match level {
        libc::SOL_SOCKET => match optname {
            libc::SO_BROADCAST => write_bool(sockopt::get_socket_broadcast(fd), optval, optlen),
            libc::SO_LINGER => write_linger(sockopt::get_socket_linger(fd), optval, optlen),
            libc::SO_PASSCRED => write_bool(sockopt::get_socket_passcred(fd), optval, optlen),
            libc::SO_SNDTIMEO => write_timeval(
                sockopt::get_socket_timeout(fd, Timeout::Send),
                optval,
                optlen,
            ),
            libc::SO_RCVTIMEO => write_timeval(
                sockopt::get_socket_timeout(fd, Timeout::Recv),
                optval,
                optlen,
            ),
            _ => unimplemented!("unimplemented getsockopt SOL_SOCKET optname {:?}", optname),
        },
        libc::IPPROTO_IP => match optname {
            libc::IP_TTL => write_u32(sockopt::get_ip_ttl(fd), optval, optlen),
            libc::IP_MULTICAST_LOOP => {
                write_bool(sockopt::get_ip_multicast_loop(fd), optval, optlen)
            }
            libc::IP_MULTICAST_TTL => write_u32(sockopt::get_ip_multicast_ttl(fd), optval, optlen),
            _ => unimplemented!("unimplemented getsockopt IPPROTO_IP optname {:?}", optname),
        },
        libc::IPPROTO_IPV6 => match optname {
            libc::IPV6_MULTICAST_LOOP => {
                write_bool(sockopt::get_ipv6_multicast_loop(fd), optval, optlen)
            }
            libc::IPV6_V6ONLY => write_bool(sockopt::get_ipv6_v6only(fd), optval, optlen),
            _ => unimplemented!(
                "unimplemented getsockopt IPPROTO_IPV6 optname {:?}",
                optname
            ),
        },
        libc::IPPROTO_TCP => match optname {
            libc::TCP_NODELAY => write_bool(sockopt::get_tcp_nodelay(fd), optval, optlen),
            _ => unimplemented!("unimplemented getsockopt IPPROTO_TCP optname {:?}", optname),
        },
        _ => unimplemented!(
            "unimplemented getsockopt level {:?} optname {:?}",
            level,
            optname
        ),
    };
    match convert_res(result) {
        Some(()) => 0,
        None => -1,
    }
}

#[cfg(feature = "net")]
#[no_mangle]
unsafe extern "C" fn setsockopt(
    fd: c_int,
    level: c_int,
    optname: c_int,
    optval: *const c_void,
    optlen: libc::socklen_t,
) -> c_int {
    use core::time::Duration;
    use rustix::net::sockopt::{self, Timeout};

    unsafe fn read_bool(optval: *const c_void, optlen: libc::socklen_t) -> bool {
        read(optval.cast::<c_int>(), optlen) != 0
    }

    unsafe fn read_u32(optval: *const c_void, optlen: libc::socklen_t) -> u32 {
        read(optval.cast::<u32>(), optlen)
    }

    unsafe fn read_linger(optval: *const c_void, optlen: libc::socklen_t) -> Option<Duration> {
        let linger = read(optval.cast::<libc::linger>(), optlen);
        (linger.l_onoff != 0).then(|| Duration::from_secs(linger.l_linger as u64))
    }

    unsafe fn read_timeval(optval: *const c_void, optlen: libc::socklen_t) -> Option<Duration> {
        let timeval = read(optval.cast::<libc::timeval>(), optlen);
        if timeval.tv_sec == 0 && timeval.tv_usec == 0 {
            None
        } else {
            Some(
                Duration::from_secs(timeval.tv_sec.try_into().unwrap())
                    + Duration::from_micros(timeval.tv_usec as _),
            )
        }
    }

    unsafe fn read_ip_multiaddr(optval: *const c_void, optlen: libc::socklen_t) -> Ipv4Addr {
        Ipv4Addr::from(
            read(optval.cast::<libc::ip_mreq>(), optlen)
                .imr_multiaddr
                .s_addr,
        )
    }

    unsafe fn read_ip_interface(optval: *const c_void, optlen: libc::socklen_t) -> Ipv4Addr {
        Ipv4Addr::from(
            read(optval.cast::<libc::ip_mreq>(), optlen)
                .imr_interface
                .s_addr,
        )
    }

    unsafe fn read_ipv6_multiaddr(optval: *const c_void, optlen: libc::socklen_t) -> Ipv6Addr {
        Ipv6Addr::from(
            read(optval.cast::<libc::ipv6_mreq>(), optlen)
                .ipv6mr_multiaddr
                .s6_addr,
        )
    }

    unsafe fn read_ipv6_interface(optval: *const c_void, optlen: libc::socklen_t) -> u32 {
        read(optval.cast::<libc::ipv6_mreq>(), optlen).ipv6mr_interface
    }

    unsafe fn read<T>(optval: *const T, optlen: libc::socklen_t) -> T {
        assert_eq!(optlen, size_of::<T>().try_into().unwrap());
        optval.read()
    }

    libc!(libc::setsockopt(fd, level, optname, optval, optlen));

    let fd = &BorrowedFd::borrow_raw_fd(fd);
    let result = match level {
        libc::SOL_SOCKET => match optname {
            libc::SO_REUSEADDR => sockopt::set_socket_reuseaddr(fd, read_bool(optval, optlen)),
            libc::SO_BROADCAST => sockopt::set_socket_broadcast(fd, read_bool(optval, optlen)),
            libc::SO_LINGER => sockopt::set_socket_linger(fd, read_linger(optval, optlen)),
            libc::SO_PASSCRED => sockopt::set_socket_passcred(fd, read_bool(optval, optlen)),
            libc::SO_SNDTIMEO => {
                sockopt::set_socket_timeout(fd, Timeout::Send, read_timeval(optval, optlen))
            }
            libc::SO_RCVTIMEO => {
                sockopt::set_socket_timeout(fd, Timeout::Recv, read_timeval(optval, optlen))
            }
            _ => unimplemented!("unimplemented setsockopt SOL_SOCKET optname {:?}", optname),
        },
        libc::IPPROTO_IP => match optname {
            libc::IP_TTL => sockopt::set_ip_ttl(fd, read_u32(optval, optlen)),
            libc::IP_MULTICAST_LOOP => {
                sockopt::set_ip_multicast_loop(fd, read_bool(optval, optlen))
            }
            libc::IP_MULTICAST_TTL => sockopt::set_ip_multicast_ttl(fd, read_u32(optval, optlen)),
            libc::IP_ADD_MEMBERSHIP => sockopt::set_ip_add_membership(
                fd,
                &read_ip_multiaddr(optval, optlen),
                &read_ip_interface(optval, optlen),
            ),
            libc::IP_DROP_MEMBERSHIP => sockopt::set_ip_add_membership(
                fd,
                &read_ip_multiaddr(optval, optlen),
                &read_ip_interface(optval, optlen),
            ),
            _ => unimplemented!("unimplemented setsockopt IPPROTO_IP optname {:?}", optname),
        },
        libc::IPPROTO_IPV6 => match optname {
            libc::IPV6_MULTICAST_LOOP => {
                sockopt::set_ipv6_multicast_loop(fd, read_bool(optval, optlen))
            }
            libc::IPV6_ADD_MEMBERSHIP => sockopt::set_ipv6_add_membership(
                fd,
                &read_ipv6_multiaddr(optval, optlen),
                read_ipv6_interface(optval, optlen),
            ),
            libc::IPV6_DROP_MEMBERSHIP => sockopt::set_ipv6_drop_membership(
                fd,
                &read_ipv6_multiaddr(optval, optlen),
                read_ipv6_interface(optval, optlen),
            ),
            libc::IPV6_V6ONLY => sockopt::set_ipv6_v6only(fd, read_bool(optval, optlen)),
            _ => unimplemented!(
                "unimplemented setsockopt IPPROTO_IPV6 optname {:?}",
                optname
            ),
        },
        libc::IPPROTO_TCP => match optname {
            libc::TCP_NODELAY => sockopt::set_tcp_nodelay(fd, read_bool(optval, optlen)),
            _ => unimplemented!("unimplemented setsockopt IPPROTO_TCP optname {:?}", optname),
        },
        _ => unimplemented!(
            "unimplemented setsockopt level {:?} optname {:?}",
            level,
            optname
        ),
    };
    match convert_res(result) {
        Some(()) => 0,
        None => -1,
    }
}

#[cfg(feature = "sync-resolve")]
#[no_mangle]
unsafe extern "C" fn getaddrinfo(
    node: *const c_char,
    service: *const c_char,
    hints: *const libc::addrinfo,
    res: *mut *mut libc::addrinfo,
) -> c_int {
    libc!(libc::getaddrinfo(
        node,
        service,
        same_ptr(hints),
        same_ptr_mut(res)
    ));

    assert!(service.is_null(), "service lookups not supported yet");
    assert!(!node.is_null(), "only name lookups are supported corrently");

    if !hints.is_null() {
        let hints = &*hints;
        assert_eq!(hints.ai_flags, 0, "GAI flags hint not supported yet");
        assert_eq!(hints.ai_family, 0, "GAI family hint not supported yet");
        assert_eq!(
            hints.ai_socktype,
            SocketType::STREAM.as_raw() as _,
            "only SOCK_STREAM supported currently"
        );
        assert_eq!(hints.ai_protocol, 0, "GAI protocl hint not supported yet");
        assert_eq!(hints.ai_addrlen, 0, "GAI addrlen hint not supported yet");
        assert!(hints.ai_addr.is_null(), "GAI addr hint not supported yet");
        assert!(
            hints.ai_canonname.is_null(),
            "GAI canonname hint not supported yet"
        );
        assert!(hints.ai_next.is_null(), "GAI next hint not supported yet");
    }

    let host = match ZStr::from_ptr(node.cast()).to_str() {
        Ok(host) => host,
        Err(_) => {
            set_errno(Errno(libc::EILSEQ));
            return libc::EAI_SYSTEM;
        }
    };

    let layout = alloc::alloc::Layout::new::<libc::addrinfo>();
    let addr_layout = alloc::alloc::Layout::new::<SocketAddrStorage>();
    let mut first: *mut libc::addrinfo = null_mut();
    let mut prev: *mut libc::addrinfo = null_mut();
    match resolve_host(host) {
        Ok(addrs) => {
            for addr in addrs {
                let ptr = alloc::alloc::alloc(layout).cast::<libc::addrinfo>();
                ptr.write(zeroed());
                let info = &mut *ptr;
                match addr {
                    IpAddr::V4(v4) => {
                        // TODO: Create and write to `SocketAddrV4Storage`?
                        let storage = alloc::alloc::alloc(addr_layout).cast::<SocketAddrStorage>();
                        let len = SocketAddrAny::V4(SocketAddrV4::new(v4, 0)).write(storage);
                        info.ai_addr = storage.cast();
                        info.ai_addrlen = len.try_into().unwrap();
                    }
                    IpAddr::V6(v6) => {
                        // TODO: Create and write to `SocketAddrV6Storage`?
                        let storage = alloc::alloc::alloc(addr_layout).cast::<SocketAddrStorage>();
                        let len = SocketAddrAny::V6(SocketAddrV6::new(v6, 0, 0, 0)).write(storage);
                        info.ai_addr = storage.cast();
                        info.ai_addrlen = len.try_into().unwrap();
                    }
                }
                if !prev.is_null() {
                    (*prev).ai_next = ptr;
                }
                prev = ptr;
                if first.is_null() {
                    first = ptr;
                }
            }
            *res = first;
            0
        }
        Err(err) => {
            if let Some(err) = rustix::io::Error::from_io_error(&err) {
                set_errno(Errno(err.raw_os_error()));
                libc::EAI_SYSTEM
            } else {
                // TODO: sync-resolve should return a custom error type
                if err.to_string()
                    == "failed to resolve host: server responded with error: server failure"
                    || err.to_string()
                        == "failed to resolve host: server responded with error: no such name"
                {
                    libc::EAI_NONAME
                } else {
                    panic!("unknown error: {}", err);
                }
            }
        }
    }
}

#[cfg(feature = "sync-resolve")]
#[no_mangle]
unsafe extern "C" fn freeaddrinfo(mut res: *mut libc::addrinfo) {
    libc!(libc::freeaddrinfo(same_ptr_mut(res)));

    let layout = alloc::alloc::Layout::new::<libc::addrinfo>();
    let addr_layout = alloc::alloc::Layout::new::<SocketAddrStorage>();

    while !res.is_null() {
        let addr = (*res).ai_addr;
        if !addr.is_null() {
            alloc::alloc::dealloc(addr.cast::<_>(), addr_layout);
        }
        let old = res;
        res = (*res).ai_next;
        alloc::alloc::dealloc(old.cast::<_>(), layout);
    }
}

#[cfg(feature = "sync-resolve")]
#[no_mangle]
unsafe extern "C" fn gai_strerror(errcode: c_int) -> *const c_char {
    libc!(libc::gai_strerror(errcode));

    match errcode {
        libc::EAI_NONAME => &b"Name does not resolve\0"[..],
        libc::EAI_SYSTEM => &b"System error\0"[..],
        _ => panic!("unrecognized gai_strerror {:?}", errcode),
    }
    .as_ptr()
    .cast::<_>()
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn gethostname(name: *mut c_char, len: usize) -> c_int {
    let uname = rustix::process::uname();
    let nodename = uname.nodename();
    if nodename.to_bytes().len() + 1 > len {
        set_errno(Errno(libc::ENAMETOOLONG));
        return -1;
    }
    memcpy(
        name.cast::<_>(),
        nodename.to_bytes().as_ptr().cast::<_>(),
        nodename.to_bytes().len(),
    );
    *name.add(nodename.to_bytes().len()) = 0;
    0
}

#[cfg(feature = "net")]
#[no_mangle]
unsafe extern "C" fn listen(fd: c_int, backlog: c_int) -> c_int {
    libc!(libc::listen(fd, backlog));

    match convert_res(rustix::net::listen(&BorrowedFd::borrow_raw_fd(fd), backlog)) {
        Some(()) => 0,
        None => -1,
    }
}

#[cfg(feature = "net")]
#[no_mangle]
unsafe extern "C" fn recv(fd: c_int, ptr: *mut c_void, len: usize, flags: c_int) -> isize {
    libc!(libc::recv(fd, ptr, len, flags));

    let flags = RecvFlags::from_bits(flags as _).unwrap();
    match convert_res(rustix::net::recv(
        &BorrowedFd::borrow_raw_fd(fd),
        slice::from_raw_parts_mut(ptr.cast::<u8>(), len),
        flags,
    )) {
        Some(nread) => nread as isize,
        None => -1,
    }
}

#[cfg(feature = "net")]
#[no_mangle]
unsafe extern "C" fn recvfrom(
    fd: c_int,
    buf: *mut c_void,
    len: usize,
    flags: c_int,
    from: *mut SocketAddrStorage,
    from_len: *mut libc::socklen_t,
) -> isize {
    libc!(libc::recvfrom(
        fd,
        buf,
        len,
        flags,
        same_ptr_mut(from),
        from_len
    ));

    let flags = RecvFlags::from_bits(flags as _).unwrap();
    match convert_res(rustix::net::recvfrom(
        &BorrowedFd::borrow_raw_fd(fd),
        slice::from_raw_parts_mut(buf.cast::<u8>(), len),
        flags,
    )) {
        Some((nread, addr)) => {
            let encoded_len = addr.write(from);
            *from_len = encoded_len.try_into().unwrap();
            nread as isize
        }
        None => -1,
    }
}

#[cfg(feature = "net")]
#[no_mangle]
unsafe extern "C" fn send(fd: c_int, buf: *const c_void, len: usize, flags: c_int) -> isize {
    libc!(libc::send(fd, buf, len, flags));

    let flags = SendFlags::from_bits(flags as _).unwrap();
    match convert_res(rustix::net::send(
        &BorrowedFd::borrow_raw_fd(fd),
        slice::from_raw_parts(buf.cast::<u8>(), len),
        flags,
    )) {
        Some(nwritten) => nwritten as isize,
        None => -1,
    }
}

#[cfg(feature = "net")]
#[no_mangle]
unsafe extern "C" fn sendto(
    fd: c_int,
    buf: *const c_void,
    len: usize,
    flags: c_int,
    to: *const SocketAddrStorage,
    to_len: libc::socklen_t,
) -> isize {
    libc!(libc::sendto(fd, buf, len, flags, same_ptr(to), to_len));

    let flags = SendFlags::from_bits(flags as _).unwrap();
    let addr = match convert_res(SocketAddrAny::read(to, to_len.try_into().unwrap())) {
        Some(addr) => addr,
        None => return -1,
    };
    match convert_res(match addr {
        SocketAddrAny::V4(v4) => rustix::net::sendto_v4(
            &BorrowedFd::borrow_raw_fd(fd),
            slice::from_raw_parts(buf.cast::<u8>(), len),
            flags,
            &v4,
        ),
        SocketAddrAny::V6(v6) => rustix::net::sendto_v6(
            &BorrowedFd::borrow_raw_fd(fd),
            slice::from_raw_parts(buf.cast::<u8>(), len),
            flags,
            &v6,
        ),
        SocketAddrAny::Unix(unix) => rustix::net::sendto_unix(
            &BorrowedFd::borrow_raw_fd(fd),
            slice::from_raw_parts(buf.cast::<u8>(), len),
            flags,
            &unix,
        ),
        _ => panic!("unrecognized SocketAddrAny kind"),
    }) {
        Some(nwritten) => nwritten as isize,
        None => -1,
    }
}

#[cfg(feature = "net")]
#[no_mangle]
unsafe extern "C" fn shutdown(fd: c_int, how: c_int) -> c_int {
    libc!(libc::shutdown(fd, how));

    let how = match how {
        libc::SHUT_RD => Shutdown::Read,
        libc::SHUT_WR => Shutdown::Write,
        libc::SHUT_RDWR => Shutdown::ReadWrite,
        _ => panic!("unrecognized shutdown kind {}", how),
    };
    match convert_res(rustix::net::shutdown(&BorrowedFd::borrow_raw_fd(fd), how)) {
        Some(()) => 0,
        None => -1,
    }
}

#[cfg(feature = "net")]
#[no_mangle]
unsafe extern "C" fn socket(domain: c_int, type_: c_int, protocol: c_int) -> c_int {
    libc!(libc::socket(domain, type_, protocol));

    let domain = AddressFamily::from_raw(domain as _);
    let flags = SocketFlags::from_bits_truncate(type_ as _);
    let type_ = SocketType::from_raw(type_ as u32 & !SocketFlags::all().bits());
    let protocol = Protocol::from_raw(protocol as _);
    match convert_res(rustix::net::socket_with(domain, type_, flags, protocol)) {
        Some(fd) => fd.into_raw_fd(),
        None => -1,
    }
}

#[cfg(feature = "net")]
#[no_mangle]
unsafe extern "C" fn socketpair(
    domain: c_int,
    type_: c_int,
    protocol: c_int,
    sv: *mut [c_int; 2],
) -> c_int {
    libc!(libc::socketpair(
        domain,
        type_,
        protocol,
        (*sv).as_mut_ptr()
    ));

    let domain = AddressFamily::from_raw(domain as _);
    let flags = SocketFlags::from_bits_truncate(type_ as _);
    let type_ = SocketType::from_raw(type_ as u32 & !SocketFlags::all().bits());
    let protocol = Protocol::from_raw(protocol as _);
    match convert_res(rustix::net::socketpair(domain, type_, flags, protocol)) {
        Some((fd0, fd1)) => {
            (*sv) = [fd0.into_raw_fd(), fd1.into_raw_fd()];
            0
        }
        None => -1,
    }
}

#[cfg(feature = "sync-resolve")]
#[no_mangle]
unsafe extern "C" fn __res_init() -> c_int {
    libc!(libc::res_init());
    0
}

// io

#[no_mangle]
unsafe extern "C" fn write(fd: c_int, ptr: *const c_void, len: usize) -> isize {
    libc!(libc::write(fd, ptr, len));

    match convert_res(rustix::io::write(
        &BorrowedFd::borrow_raw_fd(fd),
        slice::from_raw_parts(ptr.cast::<u8>(), len),
    )) {
        Some(nwritten) => nwritten as isize,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn writev(fd: c_int, iov: *const rustix::io::IoSlice, iovcnt: c_int) -> isize {
    libc!(libc::writev(fd, same_ptr(iov), iovcnt));

    if iovcnt < 0 {
        set_errno(Errno(libc::EINVAL));
        return -1;
    }

    match convert_res(rustix::io::writev(
        &BorrowedFd::borrow_raw_fd(fd),
        slice::from_raw_parts(iov, iovcnt as usize),
    )) {
        Some(nwritten) => nwritten as isize,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn read(fd: c_int, ptr: *mut c_void, len: usize) -> isize {
    libc!(libc::read(fd, ptr, len));

    match convert_res(rustix::io::read(
        &BorrowedFd::borrow_raw_fd(fd),
        slice::from_raw_parts_mut(ptr.cast::<u8>(), len),
    )) {
        Some(nread) => nread as isize,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn readv(fd: c_int, iov: *const rustix::io::IoSliceMut, iovcnt: c_int) -> isize {
    libc!(libc::readv(fd, same_ptr(iov), iovcnt));

    if iovcnt < 0 {
        set_errno(Errno(libc::EINVAL));
        return -1;
    }

    match convert_res(rustix::io::readv(
        &BorrowedFd::borrow_raw_fd(fd),
        slice::from_raw_parts(iov, iovcnt as usize),
    )) {
        Some(nread) => nread as isize,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn pread64(fd: c_int, ptr: *mut c_void, len: usize, offset: i64) -> isize {
    libc!(libc::pread64(fd, ptr, len, offset));

    match convert_res(rustix::io::pread(
        &BorrowedFd::borrow_raw_fd(fd),
        slice::from_raw_parts_mut(ptr.cast::<u8>(), len),
        offset as u64,
    )) {
        Some(nread) => nread as isize,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn pwrite64(fd: c_int, ptr: *const c_void, len: usize, offset: i64) -> isize {
    libc!(libc::pwrite64(fd, ptr, len, offset));

    match convert_res(rustix::io::pwrite(
        &BorrowedFd::borrow_raw_fd(fd),
        slice::from_raw_parts(ptr.cast::<u8>(), len),
        offset as u64,
    )) {
        Some(nwritten) => nwritten as isize,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn poll(fds: *mut rustix::io::PollFd, nfds: c_ulong, timeout: c_int) -> c_int {
    libc!(libc::poll(same_ptr_mut(fds), nfds, timeout));

    let fds = slice::from_raw_parts_mut(fds, nfds.try_into().unwrap());
    match convert_res(rustix::io::poll(fds, timeout)) {
        Some(num) => num.try_into().unwrap(),
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn close(fd: c_int) -> c_int {
    libc!(libc::close(fd));

    rustix::io::close(fd);
    0
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn dup2(fd: c_int, to: c_int) -> c_int {
    libc!(libc::dup2(fd, to));

    let to = OwnedFd::from_raw_fd(to).into();
    match convert_res(rustix::io::dup2(&BorrowedFd::borrow_raw_fd(fd), &to)) {
        Some(()) => OwnedFd::from(to).into_raw_fd(),
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn isatty(fd: c_int) -> c_int {
    libc!(libc::isatty(fd));

    if rustix::io::isatty(&BorrowedFd::borrow_raw_fd(fd)) {
        1
    } else {
        set_errno(Errno(libc::ENOTTY));
        0
    }
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn ioctl(fd: c_int, request: c_long, mut args: ...) -> c_int {
    const TCGETS: c_long = libc::TCGETS as c_long;
    const FIONBIO: c_long = libc::FIONBIO as c_long;
    const TIOCGWINSZ: c_long = libc::TIOCGWINSZ as c_long;
    match request {
        TCGETS => {
            libc!(libc::ioctl(fd, libc::TCGETS));
            let fd = BorrowedFd::borrow_raw_fd(fd);
            match convert_res(rustix::io::ioctl_tcgets(&fd)) {
                Some(x) => {
                    args.arg::<*mut rustix::io::Termios>().write(x);
                    0
                }
                None => -1,
            }
        }
        FIONBIO => {
            libc!(libc::ioctl(fd, libc::FIONBIO));
            let fd = BorrowedFd::borrow_raw_fd(fd);
            let ptr = args.arg::<*mut c_int>();
            let value = *ptr != 0;
            match convert_res(rustix::io::ioctl_fionbio(&fd, value)) {
                Some(()) => 0,
                None => -1,
            }
        }
        TIOCGWINSZ => {
            libc!(libc::ioctl(fd, libc::TIOCGWINSZ));
            let fd = BorrowedFd::borrow_raw_fd(fd);
            match convert_res(rustix::io::ioctl_tiocgwinsz(&fd)) {
                Some(x) => {
                    args.arg::<*mut rustix::io::Winsize>().write(x);
                    0
                }
                None => -1,
            }
        }
        _ => panic!("unrecognized ioctl({})", request),
    }
}

#[no_mangle]
unsafe extern "C" fn pipe2(pipefd: *mut c_int, flags: c_int) -> c_int {
    libc!(libc::pipe2(pipefd, flags));

    let flags = PipeFlags::from_bits(flags as _).unwrap();
    match convert_res(rustix::io::pipe_with(flags)) {
        Some((a, b)) => {
            *pipefd = a.into_raw_fd();
            *pipefd.add(1) = b.into_raw_fd();
            0
        }
        None => -1,
    }
}

#[cfg(any(target_os = "android", target_os = "linux"))]
#[no_mangle]
unsafe extern "C" fn sendfile() {
    //libc!(libc::sendfile());
    unimplemented!("sendfile")
}

#[cfg(feature = "net")]
#[no_mangle]
unsafe extern "C" fn sendmsg() {
    //libc!(libc::sendmsg());
    unimplemented!("sendmsg")
}

#[cfg(feature = "net")]
#[no_mangle]
unsafe extern "C" fn recvmsg() {
    //libc!(libc::recvmsg());
    unimplemented!("recvmsg")
}

#[cfg(any(target_os = "android", target_os = "linux"))]
#[no_mangle]
unsafe extern "C" fn epoll_create1(_flags: c_int) -> c_int {
    libc!(libc::epoll_create1(_flags));
    unimplemented!("epoll_create1")
}

#[cfg(any(target_os = "android", target_os = "linux"))]
#[no_mangle]
unsafe extern "C" fn epoll_ctl(
    _epfd: c_int,
    _op: c_int,
    _fd: c_int,
    _event: *mut libc::epoll_event,
) -> c_int {
    libc!(libc::epoll_ctl(_epfd, _op, _fd, same_ptr_mut(_event)));
    unimplemented!("epoll_ctl")
}

#[cfg(any(target_os = "android", target_os = "linux"))]
#[no_mangle]
unsafe extern "C" fn epoll_wait(
    _epfd: c_int,
    _events: *mut libc::epoll_event,
    _maxevents: c_int,
    _timeout: c_int,
) -> c_int {
    libc!(libc::epoll_wait(
        _epfd,
        same_ptr_mut(_events),
        _maxevents,
        _timeout
    ));
    unimplemented!("epoll_wait")
}

#[cfg(any(target_os = "android", target_os = "linux"))]
#[no_mangle]
unsafe extern "C" fn eventfd(initval: c_uint, flags: c_int) -> c_int {
    libc!(libc::eventfd(initval, flags));
    let flags = EventfdFlags::from_bits(flags.try_into().unwrap()).unwrap();
    match convert_res(rustix::io::eventfd(initval, flags)) {
        Some(fd) => fd.into_raw_fd(),
        None => -1,
    }
}

// malloc

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Tag {
    size: usize,
    align: usize,
}

/// allocates for a given layout, with a tag prepended to the allocation,
/// to keep track of said layout.
/// returns null if the allocation failed
fn tagged_alloc(type_layout: alloc::alloc::Layout) -> *mut u8 {
    if type_layout.size() == 0 {
        return core::ptr::null_mut();
    }

    let tag_layout = alloc::alloc::Layout::new::<Tag>();
    let tag = Tag {
        size: type_layout.size(),
        align: type_layout.align(),
    };
    if let Ok((total_layout, offset)) = tag_layout.extend(type_layout) {
        let total_ptr = unsafe { alloc::alloc::alloc(total_layout) };
        if total_ptr.is_null() {
            return total_ptr;
        }

        let tag_offset = offset - tag_layout.size();
        unsafe {
            total_ptr.wrapping_add(tag_offset).cast::<Tag>().write(tag);
            total_ptr.wrapping_add(offset).cast::<_>()
        }
    } else {
        core::ptr::null_mut()
    }
}

/// get the layout out of a tagged allocation
///
/// #Safety
/// the given pointer must be a non-null pointer,
/// gotten from calling `tagged_alloc`
unsafe fn get_layout(ptr: *mut u8) -> alloc::alloc::Layout {
    let tag = ptr
        .wrapping_sub(core::mem::size_of::<Tag>())
        .cast::<Tag>()
        .read();
    alloc::alloc::Layout::from_size_align_unchecked(tag.size, tag.align)
}

/// get the layout out of a tagged allocation
///
/// #Safety
/// the given pointer must be a non-null pointer,
/// gotten from calling `tagged_alloc`
unsafe fn tagged_dealloc(ptr: *mut u8) {
    let tag_layout = alloc::alloc::Layout::new::<Tag>();
    let type_layout = get_layout(ptr);
    if let Ok((total_layout, offset)) = tag_layout.extend(type_layout) {
        let total_ptr = ptr.wrapping_sub(offset);
        alloc::alloc::dealloc(total_ptr, total_layout);
    }
}

#[no_mangle]
unsafe extern "C" fn malloc(size: usize) -> *mut c_void {
    libc!(libc::malloc(size));

    // TODO: Add `max_align_t` for riscv64 to upstream libc.
    #[cfg(target_arch = "riscv64")]
    let layout = alloc::alloc::Layout::from_size_align(size, 16).unwrap();
    #[cfg(not(target_arch = "riscv64"))]
    let layout =
        alloc::alloc::Layout::from_size_align(size, align_of::<libc::max_align_t>()).unwrap();
    tagged_alloc(layout).cast::<_>()
}

#[no_mangle]
unsafe extern "C" fn realloc(old: *mut c_void, size: usize) -> *mut c_void {
    libc!(libc::realloc(old, size));

    if old.is_null() {
        malloc(size)
    } else {
        let old_layout = get_layout(old.cast::<_>());
        if old_layout.size() >= size {
            return old;
        }

        let new = malloc(size);
        memcpy(new, old, core::cmp::min(size, old_layout.size()));
        tagged_dealloc(old.cast::<_>());
        new
    }
}

#[no_mangle]
unsafe extern "C" fn calloc(nmemb: usize, size: usize) -> *mut c_void {
    libc!(libc::calloc(nmemb, size));

    let product = match nmemb.checked_mul(size) {
        Some(product) => product,
        None => {
            set_errno(Errno(libc::ENOMEM));
            return null_mut();
        }
    };

    let ptr = malloc(product);
    memset(ptr, 0, product)
}

#[no_mangle]
unsafe extern "C" fn posix_memalign(
    memptr: *mut *mut c_void,
    alignment: usize,
    size: usize,
) -> c_int {
    libc!(libc::posix_memalign(memptr, alignment, size));
    if !(alignment.is_power_of_two() && alignment % core::mem::size_of::<*const c_void>() == 0) {
        return rustix::io::Error::INVAL.raw_os_error();
    }

    let layout = alloc::alloc::Layout::from_size_align(size, alignment).unwrap();
    let ptr = tagged_alloc(layout).cast::<_>();

    *memptr = ptr;
    0
}

#[no_mangle]
unsafe extern "C" fn free(ptr: *mut c_void) {
    libc!(libc::free(ptr));

    if ptr.is_null() {
        return;
    }

    tagged_dealloc(ptr.cast::<_>());
}

// mem

#[no_mangle]
unsafe extern "C" fn memcmp(a: *const c_void, b: *const c_void, len: usize) -> c_int {
    libc!(libc::memcmp(a, b, len));

    compiler_builtins::mem::memcmp(a.cast::<_>(), b.cast::<_>(), len)
}

#[no_mangle]
unsafe extern "C" fn bcmp(a: *const c_void, b: *const c_void, len: usize) -> c_int {
    // `bcmp` is just an alias for `memcmp`.
    libc!(libc::memcmp(a, b, len));

    compiler_builtins::mem::bcmp(a.cast::<_>(), b.cast::<_>(), len)
}

#[no_mangle]
unsafe extern "C" fn memcpy(dst: *mut c_void, src: *const c_void, len: usize) -> *mut c_void {
    libc!(libc::memcpy(dst, src, len));

    compiler_builtins::mem::memcpy(dst.cast::<_>(), src.cast::<_>(), len).cast::<_>()
}

#[no_mangle]
unsafe extern "C" fn memmove(dst: *mut c_void, src: *const c_void, len: usize) -> *mut c_void {
    libc!(libc::memmove(dst, src, len));

    compiler_builtins::mem::memmove(dst.cast::<_>(), src.cast::<_>(), len).cast::<_>()
}

#[no_mangle]
unsafe extern "C" fn memset(dst: *mut c_void, fill: c_int, len: usize) -> *mut c_void {
    libc!(libc::memset(dst, fill, len));

    compiler_builtins::mem::memset(dst.cast::<_>(), fill, len).cast::<_>()
}

#[no_mangle]
unsafe extern "C" fn memchr(s: *const c_void, c: c_int, len: usize) -> *mut c_void {
    libc!(libc::memchr(s, c, len));

    let slice: &[u8] = slice::from_raw_parts(s.cast(), len);
    match memchr::memchr(c as u8, slice) {
        None => null_mut(),
        Some(i) => s.cast::<u8>().add(i) as *mut c_void,
    }
}

#[no_mangle]
unsafe extern "C" fn memrchr(s: *const c_void, c: c_int, len: usize) -> *mut c_void {
    libc!(libc::memrchr(s, c, len));

    let slice: &[u8] = slice::from_raw_parts(s.cast(), len);
    match memchr::memrchr(c as u8, slice) {
        None => null_mut(),
        Some(i) => s.cast::<u8>().add(i) as *mut c_void,
    }
}

#[no_mangle]
unsafe extern "C" fn strlen(mut s: *const c_char) -> usize {
    libc!(libc::strlen(s));

    let mut len = 0;
    while *s != b'\0' as _ {
        len += 1;
        s = s.add(1);
    }
    len
}

// mmap

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn mmap(
    addr: *mut c_void,
    length: usize,
    prot: c_int,
    flags: c_int,
    fd: c_int,
    offset: c_long,
) -> *mut c_void {
    libc!(libc::mmap(addr, length, prot, flags, fd, offset));

    let anon = flags & libc::MAP_ANONYMOUS == libc::MAP_ANONYMOUS;
    let prot = ProtFlags::from_bits(prot as _).unwrap();
    let flags = MapFlags::from_bits((flags & !libc::MAP_ANONYMOUS) as _).unwrap();
    match convert_res(if anon {
        rustix::io::mmap_anonymous(addr, length, prot, flags)
    } else {
        rustix::io::mmap(
            addr,
            length,
            prot,
            flags,
            &BorrowedFd::borrow_raw_fd(fd),
            offset as _,
        )
    }) {
        Some(ptr) => ptr,
        None => libc::MAP_FAILED,
    }
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn mmap64(
    addr: *mut c_void,
    length: usize,
    prot: c_int,
    flags: c_int,
    fd: c_int,
    offset: i64,
) -> *mut c_void {
    libc!(libc::mmap64(addr, length, prot, flags, fd, offset));

    let anon = flags & libc::MAP_ANONYMOUS == libc::MAP_ANONYMOUS;
    let prot = ProtFlags::from_bits(prot as _).unwrap();
    let flags = MapFlags::from_bits((flags & !libc::MAP_ANONYMOUS) as _).unwrap();
    match convert_res(if anon {
        rustix::io::mmap_anonymous(addr, length, prot, flags)
    } else {
        rustix::io::mmap(
            addr,
            length,
            prot,
            flags,
            &BorrowedFd::borrow_raw_fd(fd),
            offset as _,
        )
    }) {
        Some(ptr) => ptr,
        None => libc::MAP_FAILED,
    }
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn munmap(ptr: *mut c_void, len: usize) -> c_int {
    libc!(libc::munmap(ptr, len));

    match convert_res(rustix::io::munmap(ptr, len)) {
        Some(()) => 0,
        None => -1,
    }
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn mremap(
    old_address: *mut c_void,
    old_size: usize,
    new_size: usize,
    flags: c_int,
    mut args: ...
) -> *mut c_void {
    if (flags & libc::MREMAP_FIXED) == libc::MREMAP_FIXED {
        let new_address = args.arg::<*mut c_void>();
        libc!(libc::mremap(
            old_address,
            old_size,
            new_size,
            flags,
            new_address
        ));

        let flags = flags & !libc::MREMAP_FIXED;
        let flags = MremapFlags::from_bits(flags as _).unwrap();
        match convert_res(rustix::io::mremap_fixed(
            old_address,
            old_size,
            new_size,
            flags,
            new_address,
        )) {
            Some(new_address) => new_address,
            None => libc::MAP_FAILED,
        }
    } else {
        libc!(libc::mremap(old_address, old_size, new_size, flags));

        let flags = MremapFlags::from_bits(flags as _).unwrap();
        match convert_res(rustix::io::mremap(old_address, old_size, new_size, flags)) {
            Some(new_address) => new_address,
            None => libc::MAP_FAILED,
        }
    }
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn mprotect(addr: *mut c_void, length: usize, prot: c_int) -> c_int {
    libc!(libc::mprotect(addr, length, prot));

    let prot = MprotectFlags::from_bits(prot as _).unwrap();
    match convert_res(rustix::io::mprotect(addr, length, prot)) {
        Some(()) => 0,
        None => -1,
    }
}

// rand

#[cfg(any(target_os = "android", target_os = "linux"))]
#[no_mangle]
unsafe extern "C" fn getrandom(buf: *mut c_void, buflen: usize, flags: u32) -> isize {
    libc!(libc::getrandom(buf, buflen, flags));

    if buflen == 0 {
        return 0;
    }
    let flags = rustix::rand::GetRandomFlags::from_bits(flags).unwrap();
    match convert_res(rustix::rand::getrandom(
        slice::from_raw_parts_mut(buf.cast::<u8>(), buflen),
        flags,
    )) {
        Some(num) => num as isize,
        None => -1,
    }
}

// termios

#[no_mangle]
unsafe extern "C" fn tcgetattr() {
    //libc!(libc::tcgetattr());
    unimplemented!("tcgetattr")
}

#[no_mangle]
unsafe extern "C" fn tcsetattr() {
    //libc!(libc::tcsetattr());
    unimplemented!("tcsetattr")
}

#[no_mangle]
unsafe extern "C" fn cfmakeraw() {
    //libc!(libc::cfmakeraw());
    unimplemented!("cfmakeraw")
}

// process

#[no_mangle]
unsafe extern "C" fn chroot(_name: *const c_char) -> c_int {
    libc!(libc::chroot(_name));
    unimplemented!("chroot")
}

#[no_mangle]
unsafe extern "C" fn sysconf(name: c_int) -> c_long {
    libc!(libc::sysconf(name));

    match name {
        libc::_SC_PAGESIZE => rustix::process::page_size() as _,
        #[cfg(not(target_os = "wasi"))]
        libc::_SC_GETPW_R_SIZE_MAX => -1,
        // TODO: Oddly, only ever one processor seems to be online.
        #[cfg(not(target_os = "wasi"))]
        libc::_SC_NPROCESSORS_ONLN => 1,
        #[cfg(any(target_os = "android", target_os = "linux", target_os = "wasi"))]
        libc::_SC_SYMLOOP_MAX => 40,
        _ => panic!("unrecognized sysconf({})", name),
    }
}

#[no_mangle]
unsafe extern "C" fn getcwd(buf: *mut c_char, len: usize) -> *mut c_char {
    libc!(libc::getcwd(buf, len));

    match convert_res(rustix::process::getcwd(Vec::new())) {
        Some(path) => {
            let path = path.as_bytes();
            if path.len() + 1 <= len {
                memcpy(buf.cast(), path.as_ptr().cast(), path.len());
                *buf.add(path.len()) = 0;
                buf
            } else {
                set_errno(Errno(libc::ERANGE));
                null_mut()
            }
        }
        None => null_mut(),
    }
}

#[no_mangle]
unsafe extern "C" fn chdir(path: *const c_char) -> c_int {
    libc!(libc::chdir(path));

    let path = ZStr::from_ptr(path.cast());
    match convert_res(rustix::process::chdir(path)) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn getauxval(type_: c_ulong) -> c_ulong {
    libc!(libc::getauxval(type_));
    unimplemented!("unrecognized getauxval {}", type_)
}

#[cfg(target_arch = "aarch64")]
#[no_mangle]
unsafe extern "C" fn __getauxval(type_: c_ulong) -> c_ulong {
    libc!(libc::getauxval(type_));
    match type_ {
        libc::AT_HWCAP => rustix::process::linux_hwcap().0 as c_ulong,
        _ => unimplemented!("unrecognized __getauxval {}", type_),
    }
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn dl_iterate_phdr(
    callback: Option<
        unsafe extern "C" fn(
            info: *mut libc::dl_phdr_info,
            size: usize,
            data: *mut c_void,
        ) -> c_int,
    >,
    data: *mut c_void,
) -> c_int {
    extern "C" {
        static mut __executable_start: c_void;
    }

    // Disabled for now, as our `dl_phdr_info` has fewer fields than libc's.
    //libc!(libc::dl_iterate_phdr(callback, data));

    let (phdr, phnum) = rustix::runtime::exe_phdrs();
    let mut info = libc::dl_phdr_info {
        dlpi_addr: &mut __executable_start as *mut _ as c_ulong,
        dlpi_name: b"/proc/self/exe\0".as_ptr().cast(),
        dlpi_phdr: phdr.cast(),
        dlpi_phnum: phnum.try_into().unwrap(),
        ..zeroed()
    };
    callback.unwrap()(&mut info, size_of::<libc::dl_phdr_info>(), data)
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void {
    libc!(libc::dlsym(handle, symbol));

    if handle.is_null() {
        // `std` uses `dlsym` to dynamically detect feature availability; recognize
        // functions it asks for.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        if ZStr::from_ptr(symbol.cast()).to_bytes() == b"statx" {
            return statx as *mut c_void;
        }
        #[cfg(any(target_os = "android", target_os = "linux"))]
        if ZStr::from_ptr(symbol.cast()).to_bytes() == b"getrandom" {
            return getrandom as *mut c_void;
        }
        #[cfg(any(target_os = "android", target_os = "linux"))]
        if ZStr::from_ptr(symbol.cast()).to_bytes() == b"copy_file_range" {
            return copy_file_range as *mut c_void;
        }
        #[cfg(any(target_os = "android", target_os = "linux"))]
        if ZStr::from_ptr(symbol.cast()).to_bytes() == b"clone3" {
            // Let's just say we don't support this for now.
            return null_mut();
        }
        if ZStr::from_ptr(symbol.cast()).to_bytes() == b"__pthread_get_minstack" {
            // Let's just say we don't support this for now.
            return null_mut();
        }
        #[cfg(target_env = "gnu")]
        if ZStr::from_ptr(symbol.cast()).to_bytes() == b"gnu_get_libc_version" {
            return gnu_get_libc_version as *mut c_void;
        }
    }
    unimplemented!("dlsym({:?})", ZStr::from_ptr(symbol.cast()))
}

#[no_mangle]
unsafe extern "C" fn dlclose() {
    //libc!(libc::dlclose());
    unimplemented!("dlclose")
}

#[no_mangle]
unsafe extern "C" fn dlerror() {
    //libc!(libc::dlerror());
    unimplemented!("dlerror")
}

#[no_mangle]
unsafe extern "C" fn dlopen() {
    //libc!(libc::dlopen());
    unimplemented!("dlopen")
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn __tls_get_addr(p: &[usize; 2]) -> *mut c_void {
    //libc!(libc::__tls_get_addr(p));
    let [module, offset] = *p;
    // Offset 0 is the generation field, and we don't support dynamic linking,
    // so we should only sever see 1 here.
    assert_eq!(module, 1);
    origin::current_thread_tls_addr(offset)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "C" fn ___tls_get_addr() {
    //libc!(libc::___tls_get_addr());
    unimplemented!("___tls_get_addr")
}

#[no_mangle]
unsafe extern "C" fn sched_yield() -> c_int {
    libc!(libc::sched_yield());

    rustix::process::sched_yield();
    0
}

#[cfg(not(target_os = "wasi"))]
unsafe fn set_cpu(cpu: usize, cpuset: &mut libc::cpu_set_t) {
    libc::CPU_SET(cpu, cpuset)
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn sched_getaffinity(
    pid: c_int,
    cpu_set_size: usize,
    mask: *mut libc::cpu_set_t,
) -> c_int {
    libc!(libc::sched_getaffinity(pid, cpu_set_size, mask.cast::<_>()));
    let pid = rustix::process::Pid::from_raw(pid as _);
    match convert_res(rustix::process::sched_getaffinity(pid)) {
        Some(cpu_set) => {
            let mask = &mut *mask;
            (0..cpu_set_size)
                .filter(|&i| cpu_set.is_set(i))
                .for_each(|i| set_cpu(i, mask));
            0
        }
        None => -1,
    }
}

#[cfg(any(target_os = "android", target_os = "linux"))]
#[no_mangle]
unsafe extern "C" fn prctl(
    option: c_int,
    arg2: c_ulong,
    _arg3: c_ulong,
    _arg4: c_ulong,
    _arg5: c_ulong,
) -> c_int {
    libc!(libc::prctl(option, arg2, _arg3, _arg4, _arg5));
    match option {
        libc::PR_SET_NAME => {
            match convert_res(rustix::runtime::set_thread_name(ZStr::from_ptr(
                arg2 as *const _,
            ))) {
                Some(()) => 0,
                None => -1,
            }
        }
        _ => unimplemented!("unrecognized prctl op {}", option),
    }
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn setgid() {
    //libc!(libc::setgid());
    unimplemented!("setgid")
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn setgroups() {
    //libc!(libc::setgroups());
    unimplemented!("setgroups")
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn setuid() {
    //libc!(libc::setuid());
    unimplemented!("setuid")
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn getpid() -> c_int {
    libc!(libc::getpid());
    rustix::process::getpid().as_raw_nonzero().get() as _
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn getppid() -> c_int {
    libc!(libc::getppid());
    Pid::as_raw(rustix::process::getppid()) as _
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn getuid() -> c_uint {
    libc!(libc::getuid());
    rustix::process::getuid().as_raw()
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn getgid() -> c_uint {
    libc!(libc::getgid());
    rustix::process::getgid().as_raw()
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn kill(_pid: c_int, _sig: c_int) -> c_int {
    libc!(libc::kill(_pid, _sig));
    unimplemented!("kill")
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn raise(_sig: c_int) -> c_int {
    libc!(libc::raise(_sig));
    unimplemented!("raise")
}

// nss

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn getpwuid_r() {
    //libc!(libc::getpwuid_r());
    unimplemented!("getpwuid_r")
}

// signal

#[no_mangle]
unsafe extern "C" fn abort() {
    libc!(libc::abort());
    unimplemented!("abort")
}

#[no_mangle]
unsafe extern "C" fn signal(_num: c_int, _handler: usize) -> usize {
    libc!(libc::signal(_num, _handler));

    // Somehow no signals for this handler were ever delivered. How odd.
    libc::SIG_DFL
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn sigaction(_signum: c_int, _act: *const c_void, _oldact: *mut c_void) -> c_int {
    libc!(libc::sigaction(
        _signum,
        _act.cast::<_>(),
        _oldact.cast::<_>()
    ));

    // No signals for this handler were ever delivered either. What a coincidence.
    0
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn sigaltstack(_ss: *const c_void, _old_ss: *mut c_void) -> c_int {
    libc!(libc::sigaltstack(_ss.cast::<_>(), _old_ss.cast::<_>()));

    // Somehow no signals were delivered and the stack wasn't ever used. What luck.
    0
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn sigaddset(_sigset: *mut c_void, _signum: c_int) -> c_int {
    libc!(libc::sigaddset(_sigset.cast::<_>(), _signum));
    // Signal sets are not used right now.
    0
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn sigemptyset(_sigset: *mut c_void) -> c_int {
    libc!(libc::sigemptyset(_sigset.cast::<_>()));
    // Signal sets are not used right now.
    0
}

// syscall

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn syscall(number: c_long, mut args: ...) -> c_long {
    match number {
        libc::SYS_getrandom => {
            let buf = args.arg::<*mut c_void>();
            let len = args.arg::<usize>();
            let flags = args.arg::<u32>();
            libc!(libc::syscall(libc::SYS_getrandom, buf, len, flags));
            getrandom(buf, len, flags) as _
        }
        #[cfg(feature = "threads")]
        libc::SYS_futex => {
            use rustix::thread::{futex, FutexFlags, FutexOperation};

            let uaddr = args.arg::<*mut u32>();
            let futex_op = args.arg::<c_int>();
            let val = args.arg::<u32>();
            let timeout = args.arg::<*const libc::timespec>();
            let uaddr2 = args.arg::<*mut u32>();
            let val3 = args.arg::<u32>();
            libc!(libc::syscall(
                libc::SYS_futex,
                uaddr,
                futex_op,
                val,
                timeout,
                uaddr2,
                val3
            ));
            let flags = FutexFlags::from_bits_truncate(futex_op as _);
            let op = match futex_op & (!flags.bits() as i32) {
                libc::FUTEX_WAIT => FutexOperation::Wait,
                libc::FUTEX_WAKE => FutexOperation::Wake,
                libc::FUTEX_FD => FutexOperation::Fd,
                libc::FUTEX_REQUEUE => FutexOperation::Requeue,
                libc::FUTEX_CMP_REQUEUE => FutexOperation::CmpRequeue,
                libc::FUTEX_WAKE_OP => FutexOperation::WakeOp,
                libc::FUTEX_LOCK_PI => FutexOperation::LockPi,
                libc::FUTEX_UNLOCK_PI => FutexOperation::UnlockPi,
                libc::FUTEX_TRYLOCK_PI => FutexOperation::TrylockPi,
                libc::FUTEX_WAIT_BITSET => FutexOperation::WaitBitset,
                _ => unimplemented!("unrecognized futex op {}", futex_op),
            };
            let old_timespec = if timeout.is_null()
                || !matches!(op, FutexOperation::Wait | FutexOperation::WaitBitset)
            {
                zeroed()
            } else {
                ptr::read(timeout)
            };
            let new_timespec = rustix::time::Timespec {
                tv_sec: old_timespec.tv_sec.into(),
                tv_nsec: old_timespec.tv_nsec as _,
            };
            let new_timespec = if timeout.is_null() {
                null()
            } else {
                &new_timespec
            };
            match convert_res(futex(uaddr, op, flags, val, new_timespec, uaddr2, val3)) {
                Some(result) => result as _,
                None => -1,
            }
        }
        libc::SYS_clone3 => {
            // ensure std::process uses fork as fallback code on linux
            set_errno(Errno(libc::ENOSYS));
            -1
        }
        _ => unimplemented!("syscall({:?})", number),
    }
}

// posix_spawn

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn posix_spawnp() {
    //libc!(libc::posix_spawnp());
    unimplemented!("posix_spawnp");
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn posix_spawnattr_destroy(_ptr: *const c_void) -> c_int {
    //libc!(libc::posix_spawn_spawnattr_destroy(ptr));
    rustix::io::write(
        &rustix::io::stderr(),
        b"unimplemented: posix_spawn_spawnattr_destroy\n",
    )
    .ok();
    0
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn posix_spawnattr_init(_ptr: *const c_void) -> c_int {
    //libc!(libc::posix_spawnattr_init(ptr));
    rustix::io::write(
        &rustix::io::stderr(),
        b"unimplemented: posix_spawnattr_init\n",
    )
    .ok();
    0
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn posix_spawnattr_setflags() {
    //libc!(libc::posix_spawnattr_setflags());
    unimplemented!("posix_spawnattr_setflags")
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn posix_spawnattr_setsigdefault() {
    //libc!(libc::posix_spawnattr_setsigdefault());
    unimplemented!("posix_spawnattr_setsigdefault")
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn posix_spawnattr_setsigmask() {
    //libc!(libc::posix_spawnattr_setsigmask());
    unimplemented!("posix_spawnsetsigmask")
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn posix_spawn_file_actions_adddup2() {
    //libc!(libc::posix_spawn_file_actions_adddup2());
    unimplemented!("posix_spawn_file_actions_adddup2")
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn posix_spawn_file_actions_destroy(_ptr: *const c_void) -> c_int {
    //libc!(libc::posix_spawn_file_actions_destroy(ptr));
    rustix::io::write(
        &rustix::io::stderr(),
        b"unimplemented: posix_spawn_file_actions_destroy\n",
    )
    .ok();
    0
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn posix_spawn_file_actions_init(_ptr: *const c_void) -> c_int {
    //libc!(libc::posix_spawn_file_actions_init(ptr));
    rustix::io::write(
        &rustix::io::stderr(),
        b"unimplemented: posix_spawn_file_actions_init\n",
    )
    .ok();
    0
}

// time

#[no_mangle]
unsafe extern "C" fn clock_gettime(id: c_int, tp: *mut rustix::time::Timespec) -> c_int {
    libc!(libc::clock_gettime(id, same_ptr_mut(tp)));

    let id = match id {
        libc::CLOCK_MONOTONIC => rustix::time::ClockId::Monotonic,
        libc::CLOCK_REALTIME => rustix::time::ClockId::Realtime,
        _ => panic!("unimplemented clock({})", id),
    };
    *tp = rustix::time::clock_gettime(id);
    0
}

#[no_mangle]
unsafe extern "C" fn nanosleep(req: *const libc::timespec, rem: *mut libc::timespec) -> c_int {
    libc!(libc::nanosleep(same_ptr(req), same_ptr_mut(rem)));

    let req = rustix::time::Timespec {
        tv_sec: (*req).tv_sec.into(),
        tv_nsec: (*req).tv_nsec as _,
    };
    match rustix::thread::nanosleep(&req) {
        rustix::thread::NanosleepRelativeResult::Ok => 0,
        rustix::thread::NanosleepRelativeResult::Interrupted(remaining) => {
            *rem = libc::timespec {
                tv_sec: remaining.tv_sec.try_into().unwrap(),
                tv_nsec: remaining.tv_nsec as _,
            };
            set_errno(Errno(libc::EINTR));
            -1
        }
        rustix::thread::NanosleepRelativeResult::Err(err) => {
            set_errno(Errno(err.raw_os_error()));
            -1
        }
    }
}

// exec

#[cfg(any(target_os = "android", target_os = "linux"))]
#[no_mangle]
unsafe extern "C" fn execvp(file: *const c_char, args: *const *const c_char) -> c_int {
    libc!(libc::execvp(file, args));

    let cwd = rustix::fs::cwd();

    let file = ZStr::from_ptr(file);
    if file.to_bytes().contains(&b'/') {
        let err = rustix::runtime::execve(file, args.cast(), environ.cast());
        set_errno(Errno(err.raw_os_error()));
        return -1;
    }

    let path = _getenv(rustix::zstr!("PATH"));
    let path = if path.is_null() {
        rustix::zstr!("/bin:/usr/bin")
    } else {
        ZStr::from_ptr(path)
    };

    let mut access_error = false;
    for dir in path.to_bytes().split(|byte| *byte == b':') {
        // Open the directory and use `execveat` to avoid needing to
        // dynamically allocate a combined path string ourselves.
        let result = openat(
            &cwd,
            dir,
            OFlags::PATH | OFlags::DIRECTORY | OFlags::CLOEXEC | OFlags::NOCTTY,
            Mode::empty(),
        )
        .and_then::<(), _>(|dirfd| {
            let mut error = rustix::runtime::execveat(
                &dirfd,
                file,
                args.cast(),
                environ.cast(),
                AtFlags::empty(),
            );

            // If `execveat` is unsupported, emulate it with `execve`, without
            // allocating. This trusts /proc/self/fd.
            #[cfg(any(target_os = "android", target_os = "linux"))]
            if let rustix::io::Error::NOSYS = error {
                let fd = openat(
                    &dirfd,
                    file,
                    OFlags::PATH | OFlags::CLOEXEC | OFlags::NOCTTY,
                    Mode::empty(),
                )?;
                const PREFIX: &[u8] = b"/proc/self/fd/";
                const PREFIX_LEN: usize = PREFIX.len();
                let mut buf = [0_u8; PREFIX_LEN + 20 + 1];
                buf[..PREFIX_LEN].copy_from_slice(PREFIX);
                let fd_dec = rustix::path::DecInt::from_fd(&fd);
                let fd_bytes = fd_dec.as_z_str().to_bytes_with_nul();
                buf[PREFIX_LEN..PREFIX_LEN + fd_bytes.len()].copy_from_slice(fd_bytes);
                error = rustix::runtime::execve(
                    ZStr::from_bytes_with_nul_unchecked(&buf),
                    args.cast(),
                    environ.cast(),
                );
            }

            Err(error)
        });

        match result {
            Ok(()) => (),
            Err(rustix::io::Error::ACCESS) => access_error = true,
            Err(rustix::io::Error::NOENT) | Err(rustix::io::Error::NOTDIR) => {}
            Err(err) => {
                set_errno(Errno(err.raw_os_error()));
                return -1;
            }
        }
    }

    set_errno(Errno(if access_error {
        libc::EACCES
    } else {
        libc::ENOENT
    }));
    -1
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn fork() -> c_int {
    libc!(libc::fork());
    match convert_res(at_fork::fork()) {
        Some(Some(pid)) => pid.as_raw_nonzero().get() as c_int,
        Some(None) => 0,
        None => -1,
    }
}

// <https://refspecs.linuxbase.org/LSB_5.0.0/LSB-Core-generic/LSB-Core-generic/baselib---register-atfork.html>
#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn __register_atfork(
    prepare: Option<unsafe extern "C" fn()>,
    parent: Option<unsafe extern "C" fn()>,
    child: Option<unsafe extern "C" fn()>,
    _dso_handle: *mut c_void,
) -> c_int {
    //libc!(libc::__register_atfork(prepare, parent, child, _dso_handle));
    at_fork::at_fork(prepare, parent, child);
    0
}

#[no_mangle]
unsafe extern "C" fn clone3() {
    //libc!(libc::clone3());
    unimplemented!("clone3")
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn waitpid(pid: c_int, status: *mut c_int, options: c_int) -> c_int {
    libc!(libc::waitpid(pid, status, options));
    let options = WaitOptions::from_bits(options as _).unwrap();
    let ret_pid;
    let ret_status;
    match pid {
        -1 => match convert_res(rustix::process::wait(options)) {
            Some(Some((new_pid, new_status))) => {
                ret_pid = new_pid.as_raw_nonzero().get() as c_int;
                ret_status = new_status.as_raw() as c_int;
            }
            Some(None) => return 0,
            None => return -1,
        },
        pid => match convert_res(rustix::process::waitpid(
            rustix::process::Pid::from_raw(pid as _),
            options,
        )) {
            Some(Some(new_status)) => {
                ret_pid = if pid == 0 {
                    rustix::process::getpid().as_raw_nonzero().get() as c_int
                } else {
                    pid
                };
                ret_status = new_status.as_raw() as c_int;
            }
            Some(None) => return 0,
            None => return -1,
        },
    }
    if !status.is_null() {
        status.write(ret_status);
    }
    return ret_pid;
}

// setjmp

#[no_mangle]
unsafe extern "C" fn siglongjmp() {
    //libc!(libc::siglongjmp());
    unimplemented!("siglongjmp")
}

#[no_mangle]
unsafe extern "C" fn __sigsetjmp() {
    //libc!(libc::__sigsetjmp());
    unimplemented!("__sigsetjmp")
}

// math

#[no_mangle]
unsafe extern "C" fn acos(x: f64) -> f64 {
    libm::acos(x)
}

#[no_mangle]
unsafe extern "C" fn acosf(x: f32) -> f32 {
    libm::acosf(x)
}

#[no_mangle]
unsafe extern "C" fn acosh(x: f64) -> f64 {
    libm::acosh(x)
}

#[no_mangle]
unsafe extern "C" fn acoshf(x: f32) -> f32 {
    libm::acoshf(x)
}

#[no_mangle]
unsafe extern "C" fn asin(x: f64) -> f64 {
    libm::asin(x)
}

#[no_mangle]
unsafe extern "C" fn asinf(x: f32) -> f32 {
    libm::asinf(x)
}

#[no_mangle]
unsafe extern "C" fn asinh(x: f64) -> f64 {
    libm::asinh(x)
}

#[no_mangle]
unsafe extern "C" fn asinhf(x: f32) -> f32 {
    libm::asinhf(x)
}

#[no_mangle]
unsafe extern "C" fn atan(x: f64) -> f64 {
    libm::atan(x)
}

#[no_mangle]
unsafe extern "C" fn atan2(x: f64, y: f64) -> f64 {
    libm::atan2(x, y)
}

#[no_mangle]
unsafe extern "C" fn atan2f(x: f32, y: f32) -> f32 {
    libm::atan2f(x, y)
}

#[no_mangle]
unsafe extern "C" fn atanf(x: f32) -> f32 {
    libm::atanf(x)
}

#[no_mangle]
unsafe extern "C" fn atanh(x: f64) -> f64 {
    libm::atanh(x)
}

#[no_mangle]
unsafe extern "C" fn atanhf(x: f32) -> f32 {
    libm::atanhf(x)
}

#[no_mangle]
unsafe extern "C" fn cbrt(x: f64) -> f64 {
    libm::cbrt(x)
}

#[no_mangle]
unsafe extern "C" fn cbrtf(x: f32) -> f32 {
    libm::cbrtf(x)
}

#[no_mangle]
unsafe extern "C" fn ceil(x: f64) -> f64 {
    libm::ceil(x)
}

#[no_mangle]
unsafe extern "C" fn ceilf(x: f32) -> f32 {
    libm::ceilf(x)
}

#[no_mangle]
unsafe extern "C" fn copysign(x: f64, y: f64) -> f64 {
    libm::copysign(x, y)
}

#[no_mangle]
unsafe extern "C" fn copysignf(x: f32, y: f32) -> f32 {
    libm::copysignf(x, y)
}

#[no_mangle]
unsafe extern "C" fn cos(x: f64) -> f64 {
    libm::cos(x)
}

#[no_mangle]
unsafe extern "C" fn cosf(x: f32) -> f32 {
    libm::cosf(x)
}

#[no_mangle]
unsafe extern "C" fn cosh(x: f64) -> f64 {
    libm::cosh(x)
}

#[no_mangle]
unsafe extern "C" fn coshf(x: f32) -> f32 {
    libm::coshf(x)
}

#[no_mangle]
unsafe extern "C" fn erf(x: f64) -> f64 {
    libm::erf(x)
}

#[no_mangle]
unsafe extern "C" fn erfc(x: f64) -> f64 {
    libm::erfc(x)
}

#[no_mangle]
unsafe extern "C" fn erfcf(x: f32) -> f32 {
    libm::erfcf(x)
}

#[no_mangle]
unsafe extern "C" fn erff(x: f32) -> f32 {
    libm::erff(x)
}

#[no_mangle]
unsafe extern "C" fn exp(x: f64) -> f64 {
    libm::exp(x)
}

#[no_mangle]
unsafe extern "C" fn exp2(x: f64) -> f64 {
    libm::exp2(x)
}

#[no_mangle]
unsafe extern "C" fn exp2f(x: f32) -> f32 {
    libm::exp2f(x)
}

#[no_mangle]
unsafe extern "C" fn exp10(x: f64) -> f64 {
    libm::exp10(x)
}

#[no_mangle]
unsafe extern "C" fn exp10f(x: f32) -> f32 {
    libm::exp10f(x)
}

#[no_mangle]
unsafe extern "C" fn expf(x: f32) -> f32 {
    libm::expf(x)
}

#[no_mangle]
unsafe extern "C" fn expm1(x: f64) -> f64 {
    libm::expm1(x)
}

#[no_mangle]
unsafe extern "C" fn expm1f(x: f32) -> f32 {
    libm::expm1f(x)
}

#[no_mangle]
unsafe extern "C" fn fabs(x: f64) -> f64 {
    libm::fabs(x)
}

#[no_mangle]
unsafe extern "C" fn fabsf(x: f32) -> f32 {
    libm::fabsf(x)
}

#[no_mangle]
unsafe extern "C" fn fdim(x: f64, y: f64) -> f64 {
    libm::fdim(x, y)
}

#[no_mangle]
unsafe extern "C" fn fdimf(x: f32, y: f32) -> f32 {
    libm::fdimf(x, y)
}

#[no_mangle]
unsafe extern "C" fn floor(x: f64) -> f64 {
    libm::floor(x)
}

#[no_mangle]
unsafe extern "C" fn floorf(x: f32) -> f32 {
    libm::floorf(x)
}

#[no_mangle]
unsafe extern "C" fn fma(x: f64, y: f64, z: f64) -> f64 {
    libm::fma(x, y, z)
}

#[no_mangle]
unsafe extern "C" fn fmaf(x: f32, y: f32, z: f32) -> f32 {
    libm::fmaf(x, y, z)
}

#[no_mangle]
unsafe extern "C" fn fmax(x: f64, y: f64) -> f64 {
    libm::fmax(x, y)
}

#[no_mangle]
unsafe extern "C" fn fmaxf(x: f32, y: f32) -> f32 {
    libm::fmaxf(x, y)
}

#[no_mangle]
unsafe extern "C" fn fmin(x: f64, y: f64) -> f64 {
    libm::fmin(x, y)
}

#[no_mangle]
unsafe extern "C" fn fminf(x: f32, y: f32) -> f32 {
    libm::fminf(x, y)
}

#[no_mangle]
unsafe extern "C" fn fmod(x: f64, y: f64) -> f64 {
    libm::fmod(x, y)
}

#[no_mangle]
unsafe extern "C" fn fmodf(x: f32, y: f32) -> f32 {
    libm::fmodf(x, y)
}

#[no_mangle]
unsafe extern "C" fn frexp(x: f64, y: *mut i32) -> f64 {
    let (a, b) = libm::frexp(x);
    *y = b;
    a
}

#[no_mangle]
unsafe extern "C" fn frexpf(x: f32, y: *mut i32) -> f32 {
    let (a, b) = libm::frexpf(x);
    *y = b;
    a
}

#[no_mangle]
unsafe extern "C" fn hypot(x: f64, y: f64) -> f64 {
    libm::hypot(x, y)
}

#[no_mangle]
unsafe extern "C" fn hypotf(x: f32, y: f32) -> f32 {
    libm::hypotf(x, y)
}

#[no_mangle]
unsafe extern "C" fn ilogb(x: f64) -> i32 {
    libm::ilogb(x)
}

#[no_mangle]
unsafe extern "C" fn ilogbf(x: f32) -> i32 {
    libm::ilogbf(x)
}

#[no_mangle]
unsafe extern "C" fn j0(x: f64) -> f64 {
    libm::j0(x)
}

#[no_mangle]
unsafe extern "C" fn j0f(x: f32) -> f32 {
    libm::j0f(x)
}

#[no_mangle]
unsafe extern "C" fn j1(x: f64) -> f64 {
    libm::j1(x)
}

#[no_mangle]
unsafe extern "C" fn j1f(x: f32) -> f32 {
    libm::j1f(x)
}

#[no_mangle]
unsafe extern "C" fn jn(x: i32, y: f64) -> f64 {
    libm::jn(x, y)
}

#[no_mangle]
unsafe extern "C" fn jnf(x: i32, y: f32) -> f32 {
    libm::jnf(x, y)
}

#[no_mangle]
unsafe extern "C" fn ldexp(x: f64, y: i32) -> f64 {
    libm::ldexp(x, y)
}

#[no_mangle]
unsafe extern "C" fn ldexpf(x: f32, y: i32) -> f32 {
    libm::ldexpf(x, y)
}

#[no_mangle]
unsafe extern "C" fn lgamma_r(x: f64, y: *mut i32) -> f64 {
    let (a, b) = libm::lgamma_r(x);
    *y = b;
    a
}

#[no_mangle]
unsafe extern "C" fn lgammaf_r(x: f32, y: *mut i32) -> f32 {
    let (a, b) = libm::lgammaf_r(x);
    *y = b;
    a
}

#[no_mangle]
unsafe extern "C" fn log(x: f64) -> f64 {
    libm::log(x)
}

#[no_mangle]
unsafe extern "C" fn log1p(x: f64) -> f64 {
    libm::log1p(x)
}

#[no_mangle]
unsafe extern "C" fn log1pf(x: f32) -> f32 {
    libm::log1pf(x)
}

#[no_mangle]
unsafe extern "C" fn log2(x: f64) -> f64 {
    libm::log2(x)
}

#[no_mangle]
unsafe extern "C" fn log2f(x: f32) -> f32 {
    libm::log2f(x)
}

#[no_mangle]
unsafe extern "C" fn log10(x: f64) -> f64 {
    libm::log10(x)
}

#[no_mangle]
unsafe extern "C" fn log10f(x: f32) -> f32 {
    libm::log10f(x)
}

#[no_mangle]
unsafe extern "C" fn logf(x: f32) -> f32 {
    libm::logf(x)
}

#[no_mangle]
unsafe extern "C" fn modf(x: f64, y: *mut f64) -> f64 {
    let (a, b) = libm::modf(x);
    *y = b;
    a
}

#[no_mangle]
unsafe extern "C" fn modff(x: f32, y: *mut f32) -> f32 {
    let (a, b) = libm::modff(x);
    *y = b;
    a
}

#[no_mangle]
unsafe extern "C" fn nextafter(x: f64, y: f64) -> f64 {
    libm::nextafter(x, y)
}

#[no_mangle]
unsafe extern "C" fn nextafterf(x: f32, y: f32) -> f32 {
    libm::nextafterf(x, y)
}

#[no_mangle]
unsafe extern "C" fn pow(x: f64, y: f64) -> f64 {
    libm::pow(x, y)
}

#[no_mangle]
unsafe extern "C" fn powf(x: f32, y: f32) -> f32 {
    libm::powf(x, y)
}

#[no_mangle]
unsafe extern "C" fn remainder(x: f64, y: f64) -> f64 {
    libm::remainder(x, y)
}

#[no_mangle]
unsafe extern "C" fn remainderf(x: f32, y: f32) -> f32 {
    libm::remainderf(x, y)
}

#[no_mangle]
unsafe extern "C" fn remquo(x: f64, y: f64, z: *mut i32) -> f64 {
    let (a, b) = libm::remquo(x, y);
    *z = b;
    a
}

#[no_mangle]
unsafe extern "C" fn remquof(x: f32, y: f32, z: *mut i32) -> f32 {
    let (a, b) = libm::remquof(x, y);
    *z = b;
    a
}

#[no_mangle]
unsafe extern "C" fn round(x: f64) -> f64 {
    libm::round(x)
}

#[no_mangle]
unsafe extern "C" fn roundf(x: f32) -> f32 {
    libm::roundf(x)
}

#[no_mangle]
unsafe extern "C" fn scalbn(x: f64, y: i32) -> f64 {
    libm::scalbn(x, y)
}

#[no_mangle]
unsafe extern "C" fn scalbnf(x: f32, y: i32) -> f32 {
    libm::scalbnf(x, y)
}

#[no_mangle]
unsafe extern "C" fn sin(x: f64) -> f64 {
    libm::sin(x)
}

#[no_mangle]
unsafe extern "C" fn sincos(x: f64, y: *mut f64, z: *mut f64) {
    let (a, b) = libm::sincos(x);
    *y = a;
    *z = b;
}

#[no_mangle]
unsafe extern "C" fn sincosf(x: f32, y: *mut f32, z: *mut f32) {
    let (a, b) = libm::sincosf(x);
    *y = a;
    *z = b;
}

#[no_mangle]
unsafe extern "C" fn sinf(x: f32) -> f32 {
    libm::sinf(x)
}

#[no_mangle]
unsafe extern "C" fn sinh(x: f64) -> f64 {
    libm::sinh(x)
}

#[no_mangle]
unsafe extern "C" fn sinhf(x: f32) -> f32 {
    libm::sinhf(x)
}

#[no_mangle]
unsafe extern "C" fn sqrt(x: f64) -> f64 {
    libm::sqrt(x)
}

#[no_mangle]
unsafe extern "C" fn sqrtf(x: f32) -> f32 {
    libm::sqrtf(x)
}

#[no_mangle]
unsafe extern "C" fn tan(x: f64) -> f64 {
    libm::tan(x)
}

#[no_mangle]
unsafe extern "C" fn tanf(x: f32) -> f32 {
    libm::tanf(x)
}

#[no_mangle]
unsafe extern "C" fn tanh(x: f64) -> f64 {
    libm::tanh(x)
}

#[no_mangle]
unsafe extern "C" fn tanhf(x: f32) -> f32 {
    libm::tanhf(x)
}

#[no_mangle]
unsafe extern "C" fn tgamma(x: f64) -> f64 {
    libm::tgamma(x)
}

#[no_mangle]
unsafe extern "C" fn tgammaf(x: f32) -> f32 {
    libm::tgammaf(x)
}

#[no_mangle]
unsafe extern "C" fn trunc(x: f64) -> f64 {
    libm::trunc(x)
}

#[no_mangle]
unsafe extern "C" fn truncf(x: f32) -> f32 {
    libm::truncf(x)
}

#[no_mangle]
unsafe extern "C" fn y0(x: f64) -> f64 {
    libm::y0(x)
}

#[no_mangle]
unsafe extern "C" fn y0f(x: f32) -> f32 {
    libm::y0f(x)
}

#[no_mangle]
unsafe extern "C" fn y1(x: f64) -> f64 {
    libm::y1(x)
}

#[no_mangle]
unsafe extern "C" fn y1f(x: f32) -> f32 {
    libm::y1f(x)
}

#[no_mangle]
unsafe extern "C" fn yn(x: i32, y: f64) -> f64 {
    libm::yn(x, y)
}

#[no_mangle]
unsafe extern "C" fn ynf(x: i32, y: f32) -> f32 {
    libm::ynf(x, y)
}

// Environment variables

#[no_mangle]
unsafe extern "C" fn getenv(key: *const c_char) -> *mut c_char {
    libc!(libc::getenv(key));
    let key = ZStr::from_ptr(key.cast());
    _getenv(key)
}

unsafe fn _getenv(key: &ZStr) -> *mut c_char {
    let mut ptr = environ;
    loop {
        let env = *ptr;
        if env.is_null() {
            break;
        }
        let mut c = env;
        while *c != (b'=' as c_char) {
            c = c.add(1);
        }
        if key.to_bytes() == slice::from_raw_parts(env.cast::<u8>(), (c as usize) - (env as usize))
        {
            return c.add(1);
        }
        ptr = ptr.add(1);
    }
    null_mut()
}

#[no_mangle]
unsafe extern "C" fn setenv(name: *const c_char, value: *const c_char, overwrite: c_int) -> c_int {
    libc!(libc::setenv(name, value, overwrite));
    unimplemented!("setenv")
}

#[no_mangle]
unsafe extern "C" fn unsetenv(name: *const c_char) -> c_int {
    libc!(libc::unsetenv(name));
    unimplemented!("unsetenv")
}

/// GLIBC passes argc, argv, and envp to functions in .init_array, as a
/// non-standard extension. Use priority 98 so that we run before any
/// normal user-defined constructor functions and our own functions which
/// depend on `getenv` working.
#[cfg(target_env = "gnu")]
#[link_section = ".init_array.00098"]
#[used]
static INIT_ARRAY: unsafe extern "C" fn(c_int, *mut *mut c_char, *mut *mut c_char) = {
    unsafe extern "C" fn function(_argc: c_int, _argv: *mut *mut c_char, envp: *mut *mut c_char) {
        init_from_envp(envp);
    }
    function
};

/// For musl etc., assume that `__environ` is available and points to the
/// original environment from the kernel, so we can find the auxv array in
/// memory after it. Use priority 98 so that we run before any normal
/// user-defined constructor functions and our own functions which
/// depend on `getenv` working.
///
/// <https://refspecs.linuxbase.org/LSB_5.0.0/LSB-Core-generic/LSB-Core-generic/baselib---environ.html>
#[cfg(not(target_env = "gnu"))]
#[cfg(not(target_os = "wasi"))]
#[link_section = ".init_array.00098"]
#[used]
static INIT_ARRAY: unsafe extern "C" fn() = {
    unsafe extern "C" fn function() {
        extern "C" {
            static __environ: *mut *mut c_char;
        }

        init_from_envp(__environ)
    }
    function
};

#[cfg(not(target_os = "wasi"))]
fn init_from_envp(envp: *mut *mut c_char) {
    unsafe {
        environ = envp;
    }
}

#[no_mangle]
static mut environ: *mut *mut c_char = null_mut();

// Threads
#[cfg(feature = "threads")]
struct GetThreadId;

#[cfg(feature = "threads")]
unsafe impl parking_lot::lock_api::GetThreadId for GetThreadId {
    const INIT: Self = Self;

    fn nonzero_thread_id(&self) -> core::num::NonZeroUsize {
        origin::current_thread_id()
            .as_raw_nonzero()
            .try_into()
            .unwrap()
    }
}

#[cfg(feature = "threads")]
#[allow(non_camel_case_types)]
type PthreadT = c_ulong;
#[cfg(feature = "threads")]
libc_type!(PthreadT, pthread_t);

#[cfg(feature = "threads")]
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Clone)]
struct PthreadAttrT {
    stack_addr: *mut c_void,
    stack_size: usize,
    guard_size: usize,
    pad0: usize,
    pad1: usize,
    pad2: usize,
    pad3: usize,
    #[cfg(target_arch = "x86")]
    pad4: usize,
    #[cfg(target_arch = "x86")]
    pad5: usize,
}
#[cfg(feature = "threads")]
libc_type!(PthreadAttrT, pthread_attr_t);

#[cfg(feature = "threads")]
impl Default for PthreadAttrT {
    fn default() -> Self {
        Self {
            stack_addr: null_mut(),
            stack_size: origin::default_stack_size(),
            guard_size: origin::default_guard_size(),
            pad0: 0,
            pad1: 0,
            pad2: 0,
            pad3: 0,
            #[cfg(target_arch = "x86")]
            pad4: 0,
            #[cfg(target_arch = "x86")]
            pad5: 0,
        }
    }
}

#[cfg(feature = "threads")]
#[allow(non_camel_case_types)]
#[repr(C)]
union pthread_mutex_u {
    // Use a custom `RawMutex` for non-reentrant pthread mutex for now, because
    // we use `dlmalloc` or `wee_alloc` for global allocations, which use
    // mutexes, and `parking_lot`'s `RawMutex` performs global allocations.
    normal: RawMutex,
    reentrant: parking_lot::lock_api::RawReentrantMutex<parking_lot::RawMutex, GetThreadId>,
}

#[cfg(feature = "threads")]
#[allow(non_camel_case_types)]
#[repr(C)]
struct PthreadMutexT {
    kind: AtomicU32,
    u: pthread_mutex_u,
    pad0: usize,
    #[cfg(target_arch = "x86")]
    pad1: usize,
}
#[cfg(feature = "threads")]
libc_type!(PthreadMutexT, pthread_mutex_t);

#[cfg(feature = "threads")]
#[allow(non_camel_case_types)]
#[repr(C)]
struct PthreadRwlockT {
    lock: parking_lot::RawRwLock,
    exclusive: AtomicBool,
    pad0: usize,
    pad1: usize,
    pad2: usize,
    pad3: usize,
    pad4: usize,
    #[cfg(target_arch = "x86")]
    pad5: usize,
}
#[cfg(feature = "threads")]
libc_type!(PthreadRwlockT, pthread_rwlock_t);

#[cfg(feature = "threads")]
#[allow(non_camel_case_types)]
#[repr(C)]
struct PthreadMutexattrT {
    kind: AtomicU32,
}
#[cfg(feature = "threads")]
libc_type!(PthreadMutexattrT, pthread_mutexattr_t);

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_self() -> PthreadT {
    libc!(libc::pthread_self());
    origin::current_thread() as PthreadT
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_getattr_np(thread: PthreadT, attr: *mut PthreadAttrT) -> c_int {
    libc!(libc::pthread_getattr_np(thread, same_ptr_mut(attr)));
    let (stack_addr, stack_size, guard_size) = origin::thread_stack(thread as *mut Thread);
    ptr::write(
        attr,
        PthreadAttrT {
            stack_addr,
            stack_size,
            guard_size,
            pad0: 0,
            pad1: 0,
            pad2: 0,
            pad3: 0,
            #[cfg(target_arch = "x86")]
            pad4: 0,
            #[cfg(target_arch = "x86")]
            pad5: 0,
        },
    );
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_attr_init(attr: *mut PthreadAttrT) -> c_int {
    libc!(libc::pthread_attr_init(same_ptr_mut(attr)));
    ptr::write(attr, PthreadAttrT::default());
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_attr_destroy(attr: *mut PthreadAttrT) -> c_int {
    libc!(libc::pthread_attr_destroy(same_ptr_mut(attr)));
    ptr::drop_in_place(attr);
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_attr_getstack(
    attr: *const PthreadAttrT,
    stackaddr: *mut *mut c_void,
    stacksize: *mut usize,
) -> c_int {
    libc!(libc::pthread_attr_getstack(
        same_ptr(attr),
        stackaddr,
        stacksize
    ));
    *stackaddr = (*attr).stack_addr;
    *stacksize = (*attr).stack_size;
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_getspecific() -> *const c_void {
    //libc!(libc::pthread_getspecific());
    rustix::io::write(
        &rustix::io::stderr(),
        b"unimplemented: pthread_getspecific\n",
    )
    .ok();
    null()
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_key_create() -> c_int {
    //libc!(libc::pthread_key_create());
    rustix::io::write(
        &rustix::io::stderr(),
        b"unimplemented: pthread_key_create\n",
    )
    .ok();
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_key_delete() -> c_int {
    //libc!(libc::pthread_key_delete());
    rustix::io::write(
        &rustix::io::stderr(),
        b"unimplemented: pthread_key_delete\n",
    )
    .ok();
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_mutexattr_destroy(attr: *mut PthreadMutexattrT) -> c_int {
    libc!(libc::pthread_mutexattr_destroy(same_ptr_mut(attr)));
    ptr::drop_in_place(attr);
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_mutexattr_init(attr: *mut PthreadMutexattrT) -> c_int {
    libc!(libc::pthread_mutexattr_init(same_ptr_mut(attr)));
    ptr::write(
        attr,
        PthreadMutexattrT {
            kind: AtomicU32::new(libc::PTHREAD_MUTEX_DEFAULT as u32),
        },
    );
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_mutexattr_settype(attr: *mut PthreadMutexattrT, kind: c_int) -> c_int {
    libc!(libc::pthread_mutexattr_settype(same_ptr_mut(attr), kind));
    (*attr).kind = AtomicU32::new(kind as u32);
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_mutex_destroy(mutex: *mut PthreadMutexT) -> c_int {
    libc!(libc::pthread_mutex_destroy(same_ptr_mut(mutex)));
    match (*mutex).kind.load(SeqCst) as i32 {
        libc::PTHREAD_MUTEX_NORMAL => ptr::drop_in_place(&mut (*mutex).u.normal),
        libc::PTHREAD_MUTEX_RECURSIVE => ptr::drop_in_place(&mut (*mutex).u.reentrant),
        libc::PTHREAD_MUTEX_ERRORCHECK => unimplemented!("PTHREAD_MUTEX_ERRORCHECK"),
        other => unimplemented!("unsupported pthread mutex kind {}", other),
    }
    (*mutex).kind.store(!0, SeqCst);
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_mutex_init(
    mutex: *mut PthreadMutexT,
    mutexattr: *const PthreadMutexattrT,
) -> c_int {
    libc!(libc::pthread_mutex_init(
        same_ptr_mut(mutex),
        same_ptr(mutexattr)
    ));
    let kind = (*mutexattr).kind.load(SeqCst);
    match kind as i32 {
        libc::PTHREAD_MUTEX_NORMAL => ptr::write(&mut (*mutex).u.normal, RawMutex::INIT),
        libc::PTHREAD_MUTEX_RECURSIVE => ptr::write(
            &mut (*mutex).u.reentrant,
            parking_lot::lock_api::RawReentrantMutex::<parking_lot::RawMutex, GetThreadId>::INIT,
        ),
        libc::PTHREAD_MUTEX_ERRORCHECK => unimplemented!("PTHREAD_MUTEX_ERRORCHECK"),
        other => unimplemented!("unsupported pthread mutex kind {}", other),
    }
    (*mutex).kind.store(kind, SeqCst);
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_mutex_lock(mutex: *mut PthreadMutexT) -> c_int {
    libc!(libc::pthread_mutex_lock(same_ptr_mut(mutex)));
    match (*mutex).kind.load(SeqCst) as i32 {
        libc::PTHREAD_MUTEX_NORMAL => (*mutex).u.normal.lock(),
        libc::PTHREAD_MUTEX_RECURSIVE => (*mutex).u.reentrant.lock(),
        libc::PTHREAD_MUTEX_ERRORCHECK => unimplemented!("PTHREAD_MUTEX_ERRORCHECK"),
        other => unimplemented!("unsupported pthread mutex kind {}", other),
    }
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_mutex_trylock(mutex: *mut PthreadMutexT) -> c_int {
    libc!(libc::pthread_mutex_trylock(same_ptr_mut(mutex)));
    if match (*mutex).kind.load(SeqCst) as i32 {
        libc::PTHREAD_MUTEX_NORMAL => (*mutex).u.normal.try_lock(),
        libc::PTHREAD_MUTEX_RECURSIVE => (*mutex).u.reentrant.try_lock(),
        libc::PTHREAD_MUTEX_ERRORCHECK => unimplemented!("PTHREAD_MUTEX_ERRORCHECK"),
        other => unimplemented!("unsupported pthread mutex kind {}", other),
    } {
        0
    } else {
        rustix::io::Error::BUSY.raw_os_error()
    }
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_mutex_unlock(mutex: *mut PthreadMutexT) -> c_int {
    libc!(libc::pthread_mutex_unlock(same_ptr_mut(mutex)));
    match (*mutex).kind.load(SeqCst) as i32 {
        libc::PTHREAD_MUTEX_NORMAL => (*mutex).u.normal.unlock(),
        libc::PTHREAD_MUTEX_RECURSIVE => (*mutex).u.reentrant.unlock(),
        libc::PTHREAD_MUTEX_ERRORCHECK => unimplemented!("PTHREAD_MUTEX_ERRORCHECK"),
        other => unimplemented!("unsupported pthread mutex kind {}", other),
    }
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_rwlock_tryrdlock(rwlock: *mut PthreadRwlockT) -> c_int {
    libc!(libc::pthread_rwlock_tryrdlock(same_ptr_mut(rwlock)));
    let result = (*rwlock).lock.try_lock_shared();
    if result {
        (*rwlock).exclusive.store(false, SeqCst);
        0
    } else {
        rustix::io::Error::BUSY.raw_os_error()
    }
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_rwlock_trywrlock(rwlock: *mut PthreadRwlockT) -> c_int {
    libc!(libc::pthread_rwlock_trywrlock(same_ptr_mut(rwlock)));
    let result = (*rwlock).lock.try_lock_exclusive();
    if result {
        (*rwlock).exclusive.store(true, SeqCst);
        0
    } else {
        rustix::io::Error::BUSY.raw_os_error()
    }
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_rwlock_rdlock(rwlock: *mut PthreadRwlockT) -> c_int {
    libc!(libc::pthread_rwlock_rdlock(same_ptr_mut(rwlock)));
    (*rwlock).lock.lock_shared();
    (*rwlock).exclusive.store(false, SeqCst);
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_rwlock_unlock(rwlock: *mut PthreadRwlockT) -> c_int {
    libc!(libc::pthread_rwlock_unlock(same_ptr_mut(rwlock)));
    if (*rwlock).exclusive.load(SeqCst) {
        (*rwlock).lock.unlock_exclusive();
    } else {
        (*rwlock).lock.unlock_shared();
    }
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_setspecific() -> c_int {
    //libc!(libc::pthread_setspecific());
    rustix::io::write(
        &rustix::io::stderr(),
        b"unimplemented: pthread_getspecific\n",
    )
    .ok();
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_attr_getguardsize(
    attr: *const PthreadAttrT,
    guardsize: *mut usize,
) -> c_int {
    libc!(libc::pthread_attr_getguardsize(same_ptr(attr), guardsize));
    *guardsize = (*attr).guard_size;
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_attr_setguardsize(attr: *mut PthreadAttrT, guardsize: usize) -> c_int {
    // TODO: libc!(libc::pthread_attr_setguardsize(attr, guardsize));
    (*attr).guard_size = guardsize;
    0
}

#[cfg(feature = "threads")]
const SIZEOF_PTHREAD_COND_T: usize = 48;

#[cfg(feature = "threads")]
#[cfg_attr(target_arch = "x86", repr(C, align(4)))]
#[cfg_attr(not(target_arch = "x86"), repr(C, align(8)))]
struct PthreadCondT {
    inner: parking_lot::Condvar,
    pad: [u8; SIZEOF_PTHREAD_COND_T - core::mem::size_of::<parking_lot::Condvar>()],
}

#[cfg(feature = "threads")]
libc_type!(PthreadCondT, pthread_cond_t);

#[cfg(feature = "threads")]
#[cfg(any(
    target_arch = "x86",
    target_arch = "x86_64",
    target_arch = "arm",
    all(target_arch = "aarch64", target_pointer_width = "32"),
    target_arch = "riscv64",
))]
const SIZEOF_PTHREAD_CONDATTR_T: usize = 4;

#[cfg(all(target_arch = "aarch64", target_pointer_width = "64"))]
const SIZEOF_PTHREAD_CONDATTR_T: usize = 8;

#[cfg(feature = "threads")]
#[repr(C, align(4))]
struct PthreadCondattrT {
    pad: [u8; SIZEOF_PTHREAD_CONDATTR_T],
}

#[cfg(feature = "threads")]
libc_type!(PthreadCondattrT, pthread_condattr_t);

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_condattr_destroy(attr: *mut PthreadCondattrT) -> c_int {
    libc!(libc::pthread_condattr_destroy(same_ptr_mut(attr)));
    let _ = attr;
    rustix::io::write(
        &rustix::io::stderr(),
        b"unimplemented: pthread_condattr_destroy\n",
    )
    .ok();
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_condattr_init(attr: *mut PthreadCondattrT) -> c_int {
    libc!(libc::pthread_condattr_init(same_ptr_mut(attr)));
    let _ = attr;
    rustix::io::write(
        &rustix::io::stderr(),
        b"unimplemented: pthread_condattr_init\n",
    )
    .ok();
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_condattr_setclock(
    attr: *mut PthreadCondattrT,
    clock_id: c_int,
) -> c_int {
    libc!(libc::pthread_condattr_setclock(
        same_ptr_mut(attr),
        clock_id
    ));
    let _ = (attr, clock_id);
    rustix::io::write(
        &rustix::io::stderr(),
        b"unimplemented: pthread_condattr_setclock\n",
    )
    .ok();
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_cond_broadcast(cond: *mut PthreadCondT) -> c_int {
    libc!(libc::pthread_cond_broadcast(same_ptr_mut(cond)));
    let _ = cond;
    rustix::io::write(
        &rustix::io::stderr(),
        b"unimplemented: pthread_cond_broadcast\n",
    )
    .ok();
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_cond_destroy(cond: *mut PthreadCondT) -> c_int {
    libc!(libc::pthread_cond_destroy(same_ptr_mut(cond)));
    let _ = cond;
    rustix::io::write(
        &rustix::io::stderr(),
        b"unimplemented: pthread_cond_destroy\n",
    )
    .ok();
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_cond_init(
    cond: *mut PthreadCondT,
    attr: *const PthreadCondattrT,
) -> c_int {
    libc!(libc::pthread_cond_init(same_ptr_mut(cond), same_ptr(attr)));
    let _ = (cond, attr);
    rustix::io::write(&rustix::io::stderr(), b"unimplemented: pthread_cond_init\n").ok();
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_cond_signal(cond: *mut PthreadCondT) -> c_int {
    libc!(libc::pthread_cond_signal(same_ptr_mut(cond)));
    let _ = cond;
    rustix::io::write(
        &rustix::io::stderr(),
        b"unimplemented: pthread_cond_signal\n",
    )
    .ok();
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_cond_wait(cond: *mut PthreadCondT, lock: *mut PthreadMutexT) -> c_int {
    libc!(libc::pthread_cond_wait(
        same_ptr_mut(cond),
        same_ptr_mut(lock)
    ));
    let _ = (cond, lock);
    rustix::io::write(&rustix::io::stderr(), b"unimplemented: pthread_cond_wait\n").ok();
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_cond_timedwait(
    cond: *mut PthreadCondT,
    lock: *mut PthreadMutexT,
    abstime: *const libc::timespec,
) -> c_int {
    libc!(libc::pthread_cond_timedwait(
        same_ptr_mut(cond),
        same_ptr_mut(lock),
        same_ptr(abstime),
    ));
    let _ = (cond, lock, abstime);
    rustix::io::write(
        &rustix::io::stderr(),
        b"unimplemented: pthread_cond_timedwait\n",
    )
    .ok();
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_rwlock_destroy(rwlock: *mut PthreadRwlockT) -> c_int {
    libc!(libc::pthread_rwlock_destroy(same_ptr_mut(rwlock)));
    ptr::drop_in_place(rwlock);
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_rwlock_wrlock(rwlock: *mut PthreadRwlockT) -> c_int {
    libc!(libc::pthread_rwlock_wrlock(same_ptr_mut(rwlock)));
    (*rwlock).lock.lock_exclusive();
    (*rwlock).exclusive.store(true, SeqCst);
    0
}

// TODO: signals, create thread-local storage, arrange for thread-local storage
// destructors to run, free the `mmap`.
#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_create(
    pthread: *mut PthreadT,
    attr: *const PthreadAttrT,
    fn_: extern "C" fn(*mut c_void) -> *mut c_void,
    arg: *mut c_void,
) -> c_int {
    libc!(libc::pthread_create(pthread, same_ptr(attr), fn_, arg));
    let PthreadAttrT {
        stack_addr,
        stack_size,
        guard_size,
        pad0: _,
        pad1: _,
        pad2: _,
        pad3: _,
        #[cfg(target_arch = "x86")]
            pad4: _,
        #[cfg(target_arch = "x86")]
            pad5: _,
    } = if attr.is_null() {
        PthreadAttrT::default()
    } else {
        (*attr).clone()
    };
    assert!(stack_addr.is_null());

    let thread = match origin::create_thread(
        Box::new(move || {
            fn_(arg);
            Some(Box::new(arg))
        }),
        stack_size,
        guard_size,
    ) {
        Ok(thread) => thread,
        Err(e) => return e.raw_os_error(),
    };

    pthread.write(thread as PthreadT);
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_detach(pthread: PthreadT) -> c_int {
    libc!(libc::pthread_detach(pthread));
    origin::detach_thread(pthread as *mut Thread);
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_join(pthread: PthreadT, retval: *mut *mut c_void) -> c_int {
    libc!(libc::pthread_join(pthread, retval));
    assert!(
        retval.is_null(),
        "pthread_join return values not supported yet"
    );

    origin::join_thread(pthread as *mut Thread);
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_sigmask() -> c_int {
    //libc!(libc::pthread_sigmask());
    // not yet implemented, what is needed fo `std::Command` to work.
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_attr_setstacksize(attr: *mut PthreadAttrT, stacksize: usize) -> c_int {
    libc!(libc::pthread_attr_setstacksize(
        same_ptr_mut(attr),
        stacksize
    ));
    (*attr).stack_size = stacksize;
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_cancel() -> c_int {
    //libc!(libc::pthread_cancel());
    unimplemented!("pthread_cancel")
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_exit() -> c_int {
    //libc!(libc::pthread_exit());
    // As with `pthread_cancel`, `pthread_exit` may be tricky to implement.
    unimplemented!("pthread_exit")
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_cleanup_push() -> c_int {
    //libc!(libc::pthread_cleanup_push());
    unimplemented!("pthread_cleanup_push")
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_cleanup_pop() -> c_int {
    //libc!(libc::pthread_cleanup_pop());
    unimplemented!("pthread_cleanup_pop")
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_setcancelstate() -> c_int {
    //libc!(libc::pthread_setcancelstate());
    unimplemented!("pthread_setcancelstate")
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_setcanceltype() -> c_int {
    //libc!(libc::pthread_setcanceltype());
    unimplemented!("pthread_setcanceltype")
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_testcancel() -> c_int {
    //libc!(libc::pthread_testcancel());
    unimplemented!("pthread_testcancel")
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_atfork(
    prepare: Option<unsafe extern "C" fn()>,
    parent: Option<unsafe extern "C" fn()>,
    child: Option<unsafe extern "C" fn()>,
) -> c_int {
    libc!(libc::pthread_atfork(prepare, parent, child));
    at_fork::at_fork(prepare, parent, child);
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn __cxa_thread_atexit_impl(
    func: unsafe extern "C" fn(*mut c_void),
    obj: *mut c_void,
    _dso_symbol: *mut c_void,
) -> c_int {
    // TODO: libc!(libc::__cxa_thread_atexit_impl(func, obj, _dso_symbol));
    origin::at_thread_exit_raw(func, obj);
    0
}

// exit

/// Register a function to be called when `exit` is called.
///
/// This function conforms to the [LSB `__cxa_atexit`] ABI.
///
/// [LSB `__cxa_atexit`]: <https://refspecs.linuxbase.org/LSB_5.0.0/LSB-Core-generic/LSB-Core-generic/baselib---cxa-atexit.html>
#[no_mangle]
unsafe extern "C" fn __cxa_atexit(
    func: unsafe extern "C" fn(*mut c_void),
    arg: *mut c_void,
    _dso: *mut c_void,
) -> c_int {
    let arg = arg as usize;
    origin::at_exit(Box::new(move || func(arg as *mut c_void)));
    0
}

#[no_mangle]
unsafe extern "C" fn __cxa_finalize(_d: *mut c_void) {}

/// C-compatible `atexit`. Consider using `__cxa_atexit` instead.
#[no_mangle]
unsafe extern "C" fn atexit(func: extern "C" fn()) -> c_int {
    libc!(libc::atexit(func));
    origin::at_exit(Box::new(move || func()));
    0
}

/// C-compatible `exit`.
///
/// Call all the registered at-exit functions, and exit the program.
#[no_mangle]
unsafe extern "C" fn exit(status: c_int) -> ! {
    libc!(libc::exit(status));
    origin::exit(status)
}

/// POSIX-compatible `_exit`.
///
/// Just exit the process.
#[no_mangle]
unsafe extern "C" fn _exit(status: c_int) -> ! {
    libc!(libc::_exit(status));
    origin::exit_immediately(status)
}

// glibc versioning

#[no_mangle]
unsafe extern "C" fn gnu_get_libc_version() -> *const c_char {
    // We're implementing the glibc ABI, but we aren't actually glibc. This
    // is used by `std` to test for certain behaviors. For now, return 2.23
    // which is a glibc version where `posix_spawn` doesn't handle `ENOENT`
    // as std wants, so it uses `fork`+`exec` instead, since we don't yet
    // implement `posix_spawn`.
    b"2.23\0".as_ptr().cast::<_>()
}

// utilities

/// Convert a rustix `Result` into an `Option` with the error stored
/// in `errno`.
fn convert_res<T>(result: Result<T, rustix::io::Error>) -> Option<T> {
    result
        .map_err(|err| set_errno(Errno(err.raw_os_error())))
        .ok()
}
