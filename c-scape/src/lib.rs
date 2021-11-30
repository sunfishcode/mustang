#![doc = include_str!("../README.md")]
#![no_builtins] // don't let LLVM optimize our `memcpy` into a `memcpy` call
#![feature(thread_local)] // for `__errno_location`
#![feature(c_variadic)] // for `ioctl` etc.
#![feature(rustc_private)] // for compiler-builtins
#![feature(untagged_unions)] // for `PthreadMutexT`
#![feature(atomic_mut_ptr)] // for `RawMutex`

#[macro_use]
mod use_libc;

mod data;
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

use error_str::error_str;
use memoffset::offset_of;
#[cfg(feature = "threads")]
use origin::Thread;
#[cfg(feature = "threads")]
use parking_lot::lock_api::{RawMutex as _, RawRwLock};
#[cfg(feature = "threads")]
use parking_lot::RawMutex;
use rustix::fs::{cwd, openat, AtFlags, FdFlags, Mode, OFlags};
use rustix::io::{EventfdFlags, MapFlags, MprotectFlags, MremapFlags, PipeFlags, ProtFlags};
use rustix::io_lifetimes::{AsFd, BorrowedFd, OwnedFd};
use rustix::net::{
    AcceptFlags, AddressFamily, IpAddr, Ipv4Addr, Ipv6Addr, Protocol, RecvFlags, SendFlags,
    Shutdown, SocketAddrAny, SocketAddrStorage, SocketAddrV4, SocketAddrV6, SocketFlags,
    SocketType,
};
use rustix::process::WaitOptions;
use std::borrow::Cow;
use std::convert::TryInto;
use std::ffi::{c_void, CStr, CString, OsStr};
use std::mem::{size_of, zeroed};
use std::os::raw::{c_char, c_int, c_long, c_uint, c_ulong};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd};
use std::ptr::{self, null, null_mut};
use std::slice;
#[cfg(feature = "threads")]
use std::sync::atomic::Ordering::SeqCst;
#[cfg(feature = "threads")]
use std::sync::atomic::{AtomicBool, AtomicU32};
use sync_resolve::resolve_host;

// errno

#[no_mangle]
unsafe extern "C" fn __errno_location() -> *mut c_int {
    libc!(__errno_location());

    // When threads are supported, this will need to return a per-thread
    // pointer, but for now, we can just return a single static address.
    #[cfg_attr(feature = "threads", thread_local)]
    static mut ERRNO: i32 = 0;
    &mut ERRNO
}

#[no_mangle]
unsafe extern "C" fn __xpg_strerror_r(errnum: c_int, buf: *mut c_char, buflen: usize) -> c_int {
    libc!(strerror_r(errnum, buf, buflen));

    let message = match error_str(rustix::io::Error::from_raw_os_error(errnum)) {
        Some(s) => s.to_owned(),
        None => format!("Unknown error {}", errnum),
    };
    let min = std::cmp::min(buflen, message.len());
    let out = slice::from_raw_parts_mut(buf.cast::<u8>(), min);
    out.copy_from_slice(message.as_bytes());
    out[out.len() - 1] = b'\0';
    0
}

// fs

#[no_mangle]
unsafe extern "C" fn open64(pathname: *const c_char, flags: c_int, mode: c_int) -> c_int {
    libc!(open64(pathname, flags, mode));

    let flags = OFlags::from_bits(flags as _).unwrap();
    let mode = Mode::from_bits(mode as _).unwrap();
    match set_errno(openat(&cwd(), CStr::from_ptr(pathname), flags, mode)) {
        Some(fd) => fd.into_raw_fd(),
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn open() {
    //libc!(open())
    unimplemented!("open")
}

#[no_mangle]
unsafe extern "C" fn readlink(pathname: *const c_char, buf: *mut c_char, bufsiz: usize) -> isize {
    libc!(readlink(pathname, buf, bufsiz));

    let path = match set_errno(rustix::fs::readlinkat(
        &cwd(),
        CStr::from_ptr(pathname),
        Vec::new(),
    )) {
        Some(path) => path,
        None => return -1,
    };
    let bytes = path.as_bytes();
    let min = std::cmp::min(bytes.len(), bufsiz);
    slice::from_raw_parts_mut(buf.cast::<_>(), min).copy_from_slice(bytes);
    min as isize
}

#[no_mangle]
unsafe extern "C" fn stat() -> c_int {
    //libc!(stat())
    unimplemented!("stat")
}

#[no_mangle]
unsafe extern "C" fn stat64(pathname: *const c_char, stat_: *mut rustix::fs::Stat) -> c_int {
    libc!(stat64(pathname, same_ptr_mut(stat_)));

    match set_errno(rustix::fs::statat(
        &cwd(),
        CStr::from_ptr(pathname),
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
    libc!(fstat(_fd, same_ptr_mut(_stat)));
    unimplemented!("fstat")
}

#[no_mangle]
unsafe extern "C" fn fstat64(fd: c_int, stat_: *mut rustix::fs::Stat) -> c_int {
    libc!(fstat64(fd, same_ptr_mut(stat_)));

    match set_errno(rustix::fs::fstat(&BorrowedFd::borrow_raw_fd(fd))) {
        Some(r) => {
            *stat_ = r;
            0
        }
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn statx(
    dirfd_: c_int,
    path: *const c_char,
    flags: c_int,
    mask: c_uint,
    stat_: *mut rustix::fs::Statx,
) -> c_int {
    libc!(statx(dirfd_, path, flags, mask, same_ptr_mut(stat_)));

    if path.is_null() || stat_.is_null() {
        *__errno_location() = rustix::io::Error::FAULT.raw_os_error();
        return -1;
    }

    let flags = AtFlags::from_bits(flags as _).unwrap();
    let mask = rustix::fs::StatxFlags::from_bits(mask).unwrap();
    match set_errno(rustix::fs::statx(
        &BorrowedFd::borrow_raw_fd(dirfd_),
        CStr::from_ptr(path),
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
    libc!(realpath(path, resolved_path));

    match realpath_ext::realpath(
        OsStr::from_bytes(CStr::from_ptr(path).to_bytes()),
        realpath_ext::RealpathFlags::empty(),
    ) {
        Ok(path) => {
            if resolved_path.is_null() {
                let ptr = malloc(path.as_os_str().len() + 1).cast::<u8>();
                if ptr.is_null() {
                    *__errno_location() = rustix::io::Error::NOMEM.raw_os_error();
                    return null_mut();
                }
                slice::from_raw_parts_mut(ptr, path.as_os_str().len())
                    .copy_from_slice(path.as_os_str().as_bytes());
                *ptr.add(path.as_os_str().len()) = b'\0';
                ptr.cast::<c_char>()
            } else {
                memcpy(
                    resolved_path.cast::<c_void>(),
                    path.as_os_str().as_bytes().as_ptr().cast::<c_void>(),
                    path.as_os_str().len(),
                );
                *resolved_path.add(path.as_os_str().len()) = b'\0' as _;
                resolved_path
            }
        }
        Err(err) => {
            *__errno_location() = err.raw_os_error().unwrap();
            null_mut()
        }
    }
}

#[no_mangle]
unsafe extern "C" fn fcntl(fd: c_int, cmd: c_int, mut args: ...) -> c_int {
    match cmd {
        data::F_GETFL => {
            libc!(fcntl(fd, F_GETFL));
            let fd = BorrowedFd::borrow_raw_fd(fd);
            match set_errno(rustix::fs::fcntl_getfl(&fd)) {
                Some(flags) => flags.bits() as _,
                None => -1,
            }
        }
        data::F_SETFD => {
            let flags = args.arg::<c_int>();
            libc!(fcntl(fd, F_SETFD, flags));
            let fd = BorrowedFd::borrow_raw_fd(fd);
            match set_errno(rustix::fs::fcntl_setfd(
                &fd,
                FdFlags::from_bits(flags as _).unwrap(),
            )) {
                Some(()) => 0,
                None => -1,
            }
        }
        data::F_DUPFD_CLOEXEC => {
            let arg = args.arg::<c_int>();
            libc!(fcntl(fd, F_DUPFD_CLOEXEC, arg));
            let fd = BorrowedFd::borrow_raw_fd(fd);
            match set_errno(rustix::fs::fcntl_dupfd_cloexec(&fd, arg)) {
                Some(fd) => fd.into_raw_fd(),
                None => -1,
            }
        }
        _ => panic!("unrecognized fnctl({})", cmd),
    }
}

#[no_mangle]
unsafe extern "C" fn mkdir(pathname: *const c_char, mode: c_uint) -> c_int {
    libc!(mkdir(pathname, mode));

    let mode = Mode::from_bits(mode as _).unwrap();
    match set_errno(rustix::fs::mkdirat(&cwd(), CStr::from_ptr(pathname), mode)) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn fdatasync(fd: c_int) -> c_int {
    libc!(fdatasync(fd));

    match set_errno(rustix::fs::fdatasync(&BorrowedFd::borrow_raw_fd(fd))) {
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
    libc!(fstatat64(fd, pathname, same_ptr_mut(stat_), flags));

    let flags = AtFlags::from_bits(flags as _).unwrap();
    match set_errno(rustix::fs::statat(
        &BorrowedFd::borrow_raw_fd(fd),
        CStr::from_ptr(pathname),
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
    libc!(fsync(fd));

    match set_errno(rustix::fs::fdatasync(&BorrowedFd::borrow_raw_fd(fd))) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn ftruncate64(fd: c_int, length: i64) -> c_int {
    libc!(ftruncate64(fd, length));

    match set_errno(rustix::fs::ftruncate(
        &BorrowedFd::borrow_raw_fd(fd),
        length as u64,
    )) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn rename(old: *const c_char, new: *const c_char) -> c_int {
    libc!(rename(old, new));

    match set_errno(rustix::fs::renameat(
        &cwd(),
        CStr::from_ptr(old),
        &cwd(),
        CStr::from_ptr(new),
    )) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn rmdir(pathname: *const c_char) -> c_int {
    libc!(rmdir(pathname));

    match set_errno(rustix::fs::unlinkat(
        &cwd(),
        CStr::from_ptr(pathname),
        AtFlags::REMOVEDIR,
    )) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn unlink(pathname: *const c_char) -> c_int {
    libc!(unlink(pathname));

    match set_errno(rustix::fs::unlinkat(
        &cwd(),
        CStr::from_ptr(pathname),
        AtFlags::empty(),
    )) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn lseek64(fd: c_int, offset: i64, whence: c_int) -> i64 {
    libc!(lseek64(fd, offset, whence));

    let seek_from = match whence {
        data::SEEK_SET => std::io::SeekFrom::Start(offset as u64),
        data::SEEK_CUR => std::io::SeekFrom::Current(offset),
        data::SEEK_END => std::io::SeekFrom::End(offset),
        _ => panic!("unrecognized whence({})", whence),
    };
    match set_errno(rustix::fs::seek(&BorrowedFd::borrow_raw_fd(fd), seek_from)) {
        Some(offset) => offset as i64,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn lstat64(pathname: *const c_char, stat_: *mut rustix::fs::Stat) -> c_int {
    libc!(lstat64(pathname, same_ptr_mut(stat_)));

    match set_errno(rustix::fs::statat(
        &cwd(),
        CStr::from_ptr(pathname),
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
    libc!(opendir(pathname).cast::<_>());

    match set_errno(rustix::fs::openat(
        &cwd(),
        CStr::from_ptr(pathname),
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
        Mode::empty(),
    )) {
        Some(fd) => fdopendir(fd.into_raw_fd()),
        None => null_mut(),
    }
}

#[no_mangle]
unsafe extern "C" fn fdopendir(fd: c_int) -> *mut c_void {
    libc!(fdopendir(fd).cast::<_>());

    match set_errno(rustix::fs::Dir::from(OwnedFd::from_raw_fd(fd))) {
        Some(dir) => Box::into_raw(Box::new(dir)).cast::<_>(),
        None => null_mut(),
    }
}

#[no_mangle]
unsafe extern "C" fn readdir64_r(
    dir: *mut c_void,
    entry: *mut data::Dirent64,
    ptr: *mut *mut data::Dirent64,
) -> c_int {
    libc!(readdir64_r(
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
                rustix::fs::FileType::RegularFile => data::DT_REG,
                rustix::fs::FileType::Directory => data::DT_DIR,
                rustix::fs::FileType::Symlink => data::DT_LNK,
                rustix::fs::FileType::Fifo => data::DT_FIFO,
                rustix::fs::FileType::Socket => data::DT_SOCK,
                rustix::fs::FileType::CharacterDevice => data::DT_CHR,
                rustix::fs::FileType::BlockDevice => data::DT_BLK,
                rustix::fs::FileType::Unknown => data::DT_UNKNOWN,
            };
            *entry = data::Dirent64 {
                d_ino: e.ino(),
                d_off: 0, // We don't implement `seekdir` yet anyway.
                d_reclen: (offset_of!(data::Dirent64, d_name) + e.file_name().to_bytes().len() + 1)
                    .try_into()
                    .unwrap(),
                d_type: file_type,
                d_name: [0; 256],
            };
            let len = std::cmp::min(256, e.file_name().to_bytes().len());
            (*entry).d_name[..len].copy_from_slice(e.file_name().to_bytes());
            *ptr = entry;
            0
        }
        Some(Err(err)) => err.raw_os_error(),
    }
}

#[no_mangle]
unsafe extern "C" fn closedir(dir: *mut c_void) -> c_int {
    libc!(closedir(dir.cast::<_>()));

    drop(Box::<rustix::fs::Dir>::from_raw(dir.cast()));
    0
}

#[no_mangle]
unsafe extern "C" fn dirfd(dir: *mut c_void) -> c_int {
    libc!(dirfd(dir.cast::<_>()));

    let dir = dir.cast::<rustix::fs::Dir>();
    (*dir).as_fd().as_raw_fd()
}

#[no_mangle]
unsafe extern "C" fn readdir() {
    //libc!(readdir());
    unimplemented!("readdir")
}

#[no_mangle]
unsafe extern "C" fn rewinddir() {
    //libc!(rewinddir());
    unimplemented!("rewinddir")
}

#[no_mangle]
unsafe extern "C" fn scandir() {
    //libc!(scandir());
    unimplemented!("scandir")
}

#[no_mangle]
unsafe extern "C" fn seekdir() {
    //libc!(seekdir());
    unimplemented!("seekdir")
}

#[no_mangle]
unsafe extern "C" fn telldir() {
    //libc!(telldir());
    unimplemented!("telldir")
}

#[no_mangle]
unsafe extern "C" fn copy_file_range(
    fd_in: c_int,
    off_in: *mut i64,
    fd_out: c_int,
    off_out: *mut i64,
    len: usize,
    flags: c_uint,
) -> isize {
    libc!(copy_file_range(fd_in, off_in, fd_out, off_out, len, flags));

    if fd_in == -1 || fd_out == -1 {
        *__errno_location() = rustix::io::Error::BADF.raw_os_error();
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
    match set_errno(rustix::fs::copy_file_range(
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

#[no_mangle]
unsafe extern "C" fn chmod(pathname: *const c_char, mode: c_uint) -> c_int {
    libc!(chmod(pathname, mode));

    let mode = Mode::from_bits(mode as _).unwrap();
    match set_errno(rustix::fs::chmodat(&cwd(), CStr::from_ptr(pathname), mode)) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn fchmod(fd: c_int, mode: c_uint) -> c_int {
    libc!(fchmod(fd, mode));

    let mode = Mode::from_bits(mode as _).unwrap();
    match set_errno(rustix::fs::fchmod(&BorrowedFd::borrow_raw_fd(fd), mode)) {
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
    libc!(linkat(olddirfd, oldpath, newdirfd, newpath, flags));

    let flags = AtFlags::from_bits(flags as _).unwrap();
    match set_errno(rustix::fs::linkat(
        &BorrowedFd::borrow_raw_fd(olddirfd),
        CStr::from_ptr(oldpath),
        &BorrowedFd::borrow_raw_fd(newdirfd),
        CStr::from_ptr(newpath),
        flags,
    )) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn symlink(target: *const c_char, linkpath: *const c_char) -> c_int {
    libc!(symlink(target, linkpath));

    match set_errno(rustix::fs::symlinkat(
        CStr::from_ptr(target),
        &cwd(),
        CStr::from_ptr(linkpath),
    )) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn chown() {
    //libc!(chown());
    unimplemented!("chown")
}

#[no_mangle]
unsafe extern "C" fn lchown() {
    //libc!(lchown());
    unimplemented!("lchown")
}

#[no_mangle]
unsafe extern "C" fn fchown() {
    //libc!(fchown());
    unimplemented!("fchown")
}

// net

#[no_mangle]
unsafe extern "C" fn accept(
    fd: c_int,
    addr: *mut SocketAddrStorage,
    len: *mut data::SockLen,
) -> c_int {
    libc!(accept(fd, same_ptr_mut(addr), len));

    match set_errno(rustix::net::acceptfrom(&BorrowedFd::borrow_raw_fd(fd))) {
        Some((accepted_fd, from)) => {
            let encoded_len = from.write(addr);
            *len = encoded_len.try_into().unwrap();
            accepted_fd.into_raw_fd()
        }
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn accept4(
    fd: c_int,
    addr: *mut SocketAddrStorage,
    len: *mut data::SockLen,
    flags: c_int,
) -> c_int {
    libc!(accept4(fd, same_ptr_mut(addr), len, flags));

    let flags = AcceptFlags::from_bits(flags as _).unwrap();
    match set_errno(rustix::net::acceptfrom_with(
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

#[no_mangle]
unsafe extern "C" fn bind(
    sockfd: c_int,
    addr: *const SocketAddrStorage,
    len: data::SockLen,
) -> c_int {
    libc!(bind(sockfd, same_ptr(addr), len));

    let addr = match set_errno(SocketAddrAny::read(addr, len.try_into().unwrap())) {
        Some(addr) => addr,
        None => return -1,
    };
    match set_errno(match addr {
        SocketAddrAny::V4(v4) => rustix::net::bind_v4(&BorrowedFd::borrow_raw_fd(sockfd), &v4),
        SocketAddrAny::V6(v6) => rustix::net::bind_v6(&BorrowedFd::borrow_raw_fd(sockfd), &v6),
        SocketAddrAny::Unix(unix) => {
            rustix::net::bind_unix(&BorrowedFd::borrow_raw_fd(sockfd), &unix)
        }
        _ => panic!("unrecognized SocketAddrAny {:?}", addr),
    }) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn connect(
    sockfd: c_int,
    addr: *const SocketAddrStorage,
    len: data::SockLen,
) -> c_int {
    libc!(connect(sockfd, same_ptr(addr), len));

    let addr = match set_errno(SocketAddrAny::read(addr, len.try_into().unwrap())) {
        Some(addr) => addr,
        None => return -1,
    };
    match set_errno(match addr {
        SocketAddrAny::V4(v4) => rustix::net::connect_v4(&BorrowedFd::borrow_raw_fd(sockfd), &v4),
        SocketAddrAny::V6(v6) => rustix::net::connect_v6(&BorrowedFd::borrow_raw_fd(sockfd), &v6),
        SocketAddrAny::Unix(unix) => {
            rustix::net::connect_unix(&BorrowedFd::borrow_raw_fd(sockfd), &unix)
        }
        _ => panic!("unrecognized SocketAddrAny {:?}", addr),
    }) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn getpeername(
    fd: c_int,
    addr: *mut SocketAddrStorage,
    len: *mut data::SockLen,
) -> c_int {
    libc!(getpeername(fd, same_ptr_mut(addr), len));

    match set_errno(rustix::net::getpeername(&BorrowedFd::borrow_raw_fd(fd))) {
        Some(from) => {
            let encoded_len = from.write(addr);
            *len = encoded_len.try_into().unwrap();
            0
        }
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn getsockname(
    fd: c_int,
    addr: *mut SocketAddrStorage,
    len: *mut data::SockLen,
) -> c_int {
    libc!(getsockname(fd, same_ptr_mut(addr), len));

    match set_errno(rustix::net::getsockname(&BorrowedFd::borrow_raw_fd(fd))) {
        Some(from) => {
            let encoded_len = from.write(addr);
            *len = encoded_len.try_into().unwrap();
            0
        }
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn getsockopt(
    fd: c_int,
    level: c_int,
    optname: c_int,
    optval: *mut c_void,
    optlen: *mut data::SockLen,
) -> c_int {
    use rustix::net::sockopt::{self, Timeout};
    use std::time::Duration;

    unsafe fn write_bool(
        value: rustix::io::Result<bool>,
        optval: *mut c_void,
        optlen: *mut data::SockLen,
    ) -> rustix::io::Result<()> {
        Ok(write(value? as c_uint, optval.cast::<c_uint>(), optlen))
    }

    unsafe fn write_u32(
        value: rustix::io::Result<u32>,
        optval: *mut c_void,
        optlen: *mut data::SockLen,
    ) -> rustix::io::Result<()> {
        Ok(write(value?, optval.cast::<u32>(), optlen))
    }

    unsafe fn write_linger(
        linger: rustix::io::Result<Option<Duration>>,
        optval: *mut c_void,
        optlen: *mut data::SockLen,
    ) -> rustix::io::Result<()> {
        let linger = linger?;
        let linger = data::Linger {
            l_onoff: linger.is_some() as c_int,
            l_linger: linger.unwrap_or_default().as_secs() as c_int,
        };
        Ok(write(linger, optval.cast::<data::Linger>(), optlen))
    }

    unsafe fn write_timeval(
        value: rustix::io::Result<Option<Duration>>,
        optval: *mut c_void,
        optlen: *mut data::SockLen,
    ) -> rustix::io::Result<()> {
        let timeval = match value? {
            None => data::OldTimeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            Some(duration) => data::OldTimeval {
                tv_sec: duration
                    .as_secs()
                    .try_into()
                    .map_err(|_| rustix::io::Error::OVERFLOW)?,
                tv_usec: duration.subsec_micros() as _,
            },
        };
        Ok(write(timeval, optval.cast::<data::OldTimeval>(), optlen))
    }

    unsafe fn write<T>(value: T, optval: *mut T, optlen: *mut data::SockLen) {
        *optlen = size_of::<T>().try_into().unwrap();
        optval.write(value)
    }

    libc!(getsockopt(fd, level, optname, optval, optlen));

    let fd = &BorrowedFd::borrow_raw_fd(fd);
    let result = match level {
        data::SOL_SOCKET => match optname {
            data::SO_BROADCAST => write_bool(sockopt::get_socket_broadcast(fd), optval, optlen),
            data::SO_LINGER => write_linger(sockopt::get_socket_linger(fd), optval, optlen),
            data::SO_PASSCRED => write_bool(sockopt::get_socket_passcred(fd), optval, optlen),
            data::SO_SNDTIMEO => write_timeval(
                sockopt::get_socket_timeout(fd, Timeout::Send),
                optval,
                optlen,
            ),
            data::SO_RCVTIMEO => write_timeval(
                sockopt::get_socket_timeout(fd, Timeout::Recv),
                optval,
                optlen,
            ),
            _ => unimplemented!("unimplemented getsockopt SOL_SOCKET optname {:?}", optname),
        },
        data::IPPROTO_IP => match optname {
            data::IP_TTL => write_u32(sockopt::get_ip_ttl(fd), optval, optlen),
            data::IP_MULTICAST_LOOP => {
                write_bool(sockopt::get_ip_multicast_loop(fd), optval, optlen)
            }
            data::IP_MULTICAST_TTL => write_u32(sockopt::get_ip_multicast_ttl(fd), optval, optlen),
            _ => unimplemented!("unimplemented getsockopt IPPROTO_IP optname {:?}", optname),
        },
        data::IPPROTO_IPV6 => match optname {
            data::IPV6_MULTICAST_LOOP => {
                write_bool(sockopt::get_ipv6_multicast_loop(fd), optval, optlen)
            }
            data::IPV6_V6ONLY => write_bool(sockopt::get_ipv6_v6only(fd), optval, optlen),
            _ => unimplemented!(
                "unimplemented getsockopt IPPROTO_IPV6 optname {:?}",
                optname
            ),
        },
        data::IPPROTO_TCP => match optname {
            data::TCP_NODELAY => write_bool(sockopt::get_tcp_nodelay(fd), optval, optlen),
            _ => unimplemented!("unimplemented getsockopt IPPROTO_TCP optname {:?}", optname),
        },
        _ => unimplemented!(
            "unimplemented getsockopt level {:?} optname {:?}",
            level,
            optname
        ),
    };
    match set_errno(result) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn setsockopt(
    fd: c_int,
    level: c_int,
    optname: c_int,
    optval: *const c_void,
    optlen: data::SockLen,
) -> c_int {
    use rustix::net::sockopt::{self, Timeout};
    use std::time::Duration;

    unsafe fn read_bool(optval: *const c_void, optlen: data::SockLen) -> bool {
        read(optval.cast::<c_int>(), optlen) != 0
    }

    unsafe fn read_u32(optval: *const c_void, optlen: data::SockLen) -> u32 {
        read(optval.cast::<u32>(), optlen)
    }

    unsafe fn read_linger(optval: *const c_void, optlen: data::SockLen) -> Option<Duration> {
        let linger = read(optval.cast::<data::Linger>(), optlen);
        (linger.l_onoff != 0).then(|| Duration::from_secs(linger.l_linger as u64))
    }

    unsafe fn read_timeval(optval: *const c_void, optlen: data::SockLen) -> Option<Duration> {
        let timeval = read(optval.cast::<data::OldTimeval>(), optlen);
        if timeval.tv_sec == 0 && timeval.tv_usec == 0 {
            None
        } else {
            Some(
                Duration::from_secs(timeval.tv_sec.try_into().unwrap())
                    + Duration::from_micros(timeval.tv_usec as _),
            )
        }
    }

    unsafe fn read_ip_multiaddr(optval: *const c_void, optlen: data::SockLen) -> Ipv4Addr {
        Ipv4Addr::from(
            read(optval.cast::<data::Ipv4Mreq>(), optlen)
                .imr_multiaddr
                .s_addr,
        )
    }

    unsafe fn read_ip_interface(optval: *const c_void, optlen: data::SockLen) -> Ipv4Addr {
        Ipv4Addr::from(
            read(optval.cast::<data::Ipv4Mreq>(), optlen)
                .imr_interface
                .s_addr,
        )
    }

    unsafe fn read_ipv6_multiaddr(optval: *const c_void, optlen: data::SockLen) -> Ipv6Addr {
        Ipv6Addr::from(
            read(optval.cast::<data::Ipv6Mreq>(), optlen)
                .ipv6mr_multiaddr
                .u6_addr8,
        )
    }

    unsafe fn read_ipv6_interface(optval: *const c_void, optlen: data::SockLen) -> u32 {
        let t: i32 = read(optval.cast::<data::Ipv6Mreq>(), optlen).ipv6mr_interface;
        t as u32
    }

    unsafe fn read<T>(optval: *const T, optlen: data::SockLen) -> T {
        assert_eq!(optlen, size_of::<T>().try_into().unwrap());
        optval.read()
    }

    libc!(setsockopt(fd, level, optname, optval, optlen));

    let fd = &BorrowedFd::borrow_raw_fd(fd);
    let result = match level {
        data::SOL_SOCKET => match optname {
            data::SO_REUSEADDR => sockopt::set_socket_reuseaddr(fd, read_bool(optval, optlen)),
            data::SO_BROADCAST => sockopt::set_socket_broadcast(fd, read_bool(optval, optlen)),
            data::SO_LINGER => sockopt::set_socket_linger(fd, read_linger(optval, optlen)),
            data::SO_PASSCRED => sockopt::set_socket_passcred(fd, read_bool(optval, optlen)),
            data::SO_SNDTIMEO => {
                sockopt::set_socket_timeout(fd, Timeout::Send, read_timeval(optval, optlen))
            }
            data::SO_RCVTIMEO => {
                sockopt::set_socket_timeout(fd, Timeout::Recv, read_timeval(optval, optlen))
            }
            _ => unimplemented!("unimplemented setsockopt SOL_SOCKET optname {:?}", optname),
        },
        data::IPPROTO_IP => match optname {
            data::IP_TTL => sockopt::set_ip_ttl(fd, read_u32(optval, optlen)),
            data::IP_MULTICAST_LOOP => {
                sockopt::set_ip_multicast_loop(fd, read_bool(optval, optlen))
            }
            data::IP_MULTICAST_TTL => sockopt::set_ip_multicast_ttl(fd, read_u32(optval, optlen)),
            data::IP_ADD_MEMBERSHIP => sockopt::set_ip_add_membership(
                fd,
                &read_ip_multiaddr(optval, optlen),
                &read_ip_interface(optval, optlen),
            ),
            data::IP_DROP_MEMBERSHIP => sockopt::set_ip_add_membership(
                fd,
                &read_ip_multiaddr(optval, optlen),
                &read_ip_interface(optval, optlen),
            ),
            _ => unimplemented!("unimplemented setsockopt IPPROTO_IP optname {:?}", optname),
        },
        data::IPPROTO_IPV6 => match optname {
            data::IPV6_MULTICAST_LOOP => {
                sockopt::set_ipv6_multicast_loop(fd, read_bool(optval, optlen))
            }
            data::IPV6_ADD_MEMBERSHIP => sockopt::set_ipv6_join_group(
                fd,
                &read_ipv6_multiaddr(optval, optlen),
                read_ipv6_interface(optval, optlen),
            ),
            data::IPV6_DROP_MEMBERSHIP => sockopt::set_ipv6_leave_group(
                fd,
                &read_ipv6_multiaddr(optval, optlen),
                read_ipv6_interface(optval, optlen),
            ),
            data::IPV6_V6ONLY => sockopt::set_ipv6_v6only(fd, read_bool(optval, optlen)),
            _ => unimplemented!(
                "unimplemented setsockopt IPPROTO_IPV6 optname {:?}",
                optname
            ),
        },
        data::IPPROTO_TCP => match optname {
            data::TCP_NODELAY => sockopt::set_tcp_nodelay(fd, read_bool(optval, optlen)),
            _ => unimplemented!("unimplemented setsockopt IPPROTO_TCP optname {:?}", optname),
        },
        _ => unimplemented!(
            "unimplemented setsockopt level {:?} optname {:?}",
            level,
            optname
        ),
    };
    match set_errno(result) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn getaddrinfo(
    node: *const c_char,
    service: *const c_char,
    hints: *const data::Addrinfo,
    res: *mut *mut data::Addrinfo,
) -> c_int {
    libc!(getaddrinfo(
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

    let host = match CStr::from_ptr(node).to_str() {
        Ok(host) => host,
        Err(_) => {
            *__errno_location() = rustix::io::Error::ILSEQ.raw_os_error();
            return data::EAI_SYSTEM;
        }
    };

    let layout = std::alloc::Layout::new::<data::Addrinfo>();
    let addr_layout = std::alloc::Layout::new::<SocketAddrStorage>();
    let mut first: *mut data::Addrinfo = null_mut();
    let mut prev: *mut data::Addrinfo = null_mut();
    match resolve_host(host) {
        Ok(addrs) => {
            for addr in addrs {
                let ptr = std::alloc::alloc(layout).cast::<data::Addrinfo>();
                ptr.write(zeroed());
                let info = &mut *ptr;
                match addr {
                    IpAddr::V4(v4) => {
                        // TODO: Create and write to `SocketAddrV4Storage`?
                        let storage = std::alloc::alloc(addr_layout).cast::<SocketAddrStorage>();
                        let len = SocketAddrAny::V4(SocketAddrV4::new(v4, 0)).write(storage);
                        info.ai_addr = storage;
                        info.ai_addrlen = len.try_into().unwrap();
                    }
                    IpAddr::V6(v6) => {
                        // TODO: Create and write to `SocketAddrV6Storage`?
                        let storage = std::alloc::alloc(addr_layout).cast::<SocketAddrStorage>();
                        let len = SocketAddrAny::V6(SocketAddrV6::new(v6, 0, 0, 0)).write(storage);
                        info.ai_addr = storage;
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
                *__errno_location() = err.raw_os_error();
                data::EAI_SYSTEM
            } else {
                // TODO: sync-resolve should return a custom error type
                if err.to_string()
                    == "failed to resolve host: server responded with error: server failure"
                    || err.to_string()
                        == "failed to resolve host: server responded with error: no such name"
                {
                    data::EAI_NONAME
                } else {
                    panic!("unknown error: {}", err);
                }
            }
        }
    }
}

#[no_mangle]
unsafe extern "C" fn freeaddrinfo(mut res: *mut data::Addrinfo) {
    libc!(freeaddrinfo(same_ptr_mut(res)));

    let layout = std::alloc::Layout::new::<data::Addrinfo>();
    let addr_layout = std::alloc::Layout::new::<SocketAddrStorage>();

    while !res.is_null() {
        let addr = (*res).ai_addr;
        if !addr.is_null() {
            std::alloc::dealloc(addr.cast::<_>(), addr_layout);
        }
        let old = res;
        res = (*res).ai_next;
        std::alloc::dealloc(old.cast::<_>(), layout);
    }
}

#[no_mangle]
unsafe extern "C" fn gai_strerror(errcode: c_int) -> *const c_char {
    libc!(gai_strerror(errcode));

    match errcode {
        data::EAI_NONAME => &b"Name does not resolve\0"[..],
        data::EAI_SYSTEM => &b"System error\0"[..],
        _ => panic!("unrecognized gai_strerror {:?}", errcode),
    }
    .as_ptr()
    .cast::<_>()
}

#[no_mangle]
unsafe extern "C" fn gethostname(name: *mut c_char, len: usize) -> c_int {
    let uname = rustix::process::uname();
    let nodename = uname.nodename();
    if nodename.to_bytes().len() + 1 > len {
        *__errno_location() = rustix::io::Error::NAMETOOLONG.raw_os_error();
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

#[no_mangle]
unsafe extern "C" fn listen(fd: c_int, backlog: c_int) -> c_int {
    libc!(listen(fd, backlog));

    match set_errno(rustix::net::listen(&BorrowedFd::borrow_raw_fd(fd), backlog)) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn recv(fd: c_int, ptr: *mut c_void, len: usize, flags: c_int) -> isize {
    libc!(recv(fd, ptr, len, flags));

    let flags = RecvFlags::from_bits(flags as _).unwrap();
    match set_errno(rustix::net::recv(
        &BorrowedFd::borrow_raw_fd(fd),
        slice::from_raw_parts_mut(ptr.cast::<u8>(), len),
        flags,
    )) {
        Some(nread) => nread as isize,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn recvfrom(
    fd: c_int,
    buf: *mut c_void,
    len: usize,
    flags: c_int,
    from: *mut SocketAddrStorage,
    from_len: *mut data::SockLen,
) -> isize {
    libc!(recvfrom(fd, buf, len, flags, same_ptr_mut(from), from_len));

    let flags = RecvFlags::from_bits(flags as _).unwrap();
    match set_errno(rustix::net::recvfrom(
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

#[no_mangle]
unsafe extern "C" fn send(fd: c_int, buf: *const c_void, len: usize, flags: c_int) -> isize {
    libc!(send(fd, buf, len, flags));

    let flags = SendFlags::from_bits(flags as _).unwrap();
    match set_errno(rustix::net::send(
        &BorrowedFd::borrow_raw_fd(fd),
        slice::from_raw_parts(buf.cast::<u8>(), len),
        flags,
    )) {
        Some(nwritten) => nwritten as isize,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn sendto(
    fd: c_int,
    buf: *const c_void,
    len: usize,
    flags: c_int,
    to: *const SocketAddrStorage,
    to_len: data::SockLen,
) -> isize {
    libc!(sendto(fd, buf, len, flags, same_ptr(to), to_len));

    let flags = SendFlags::from_bits(flags as _).unwrap();
    let addr = match set_errno(SocketAddrAny::read(to, to_len.try_into().unwrap())) {
        Some(addr) => addr,
        None => return -1,
    };
    match set_errno(match addr {
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
        _ => panic!("unrecognized SocketAddrAny {:?}", addr),
    }) {
        Some(nwritten) => nwritten as isize,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn shutdown(fd: c_int, how: c_int) -> c_int {
    libc!(shutdown(fd, how));

    let how = match how {
        data::SHUT_RD => Shutdown::Read,
        data::SHUT_WR => Shutdown::Write,
        data::SHUT_RDWR => Shutdown::ReadWrite,
        _ => panic!("unrecognized shutdown kind {}", how),
    };
    match set_errno(rustix::net::shutdown(&BorrowedFd::borrow_raw_fd(fd), how)) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn socket(domain: c_int, type_: c_int, protocol: c_int) -> c_int {
    libc!(socket(domain, type_, protocol));

    let domain = AddressFamily::from_raw(domain as _);
    let flags = SocketFlags::from_bits_truncate(type_ as _);
    let type_ = SocketType::from_raw(type_ as u32 & !SocketFlags::all().bits());
    let protocol = Protocol::from_raw(protocol as _);
    match set_errno(rustix::net::socket_with(domain, type_, flags, protocol)) {
        Some(fd) => fd.into_raw_fd(),
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn socketpair(
    domain: c_int,
    type_: c_int,
    protocol: c_int,
    sv: *mut [c_int; 2],
) -> c_int {
    libc!(socketpair(domain, type_, protocol, (*sv).as_mut_ptr()));

    let domain = AddressFamily::from_raw(domain as _);
    let flags = SocketFlags::from_bits_truncate(type_ as _);
    let type_ = SocketType::from_raw(type_ as u32 & !SocketFlags::all().bits());
    let protocol = Protocol::from_raw(protocol as _);
    match set_errno(rustix::net::socketpair(domain, type_, flags, protocol)) {
        Some((fd0, fd1)) => {
            (*sv) = [fd0.into_raw_fd(), fd1.into_raw_fd()];
            0
        }
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn __res_init() -> c_int {
    libc!(res_init());

    unimplemented!("__res_init")
}

// io

#[no_mangle]
unsafe extern "C" fn write(fd: c_int, ptr: *const c_void, len: usize) -> isize {
    libc!(write(fd, ptr, len));

    match set_errno(rustix::io::write(
        &BorrowedFd::borrow_raw_fd(fd),
        slice::from_raw_parts(ptr.cast::<u8>(), len),
    )) {
        Some(nwritten) => nwritten as isize,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn writev(fd: c_int, iov: *const std::io::IoSlice, iovcnt: c_int) -> isize {
    libc!(writev(fd, same_ptr(iov), iovcnt));

    if iovcnt < 0 {
        *__errno_location() = rustix::io::Error::INVAL.raw_os_error();
        return -1;
    }

    match set_errno(rustix::io::writev(
        &BorrowedFd::borrow_raw_fd(fd),
        slice::from_raw_parts(iov, iovcnt as usize),
    )) {
        Some(nwritten) => nwritten as isize,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn read(fd: c_int, ptr: *mut c_void, len: usize) -> isize {
    libc!(read(fd, ptr, len));

    match set_errno(rustix::io::read(
        &BorrowedFd::borrow_raw_fd(fd),
        slice::from_raw_parts_mut(ptr.cast::<u8>(), len),
    )) {
        Some(nread) => nread as isize,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn readv(fd: c_int, iov: *const std::io::IoSliceMut, iovcnt: c_int) -> isize {
    libc!(readv(fd, same_ptr(iov), iovcnt));

    if iovcnt < 0 {
        *__errno_location() = rustix::io::Error::INVAL.raw_os_error();
        return -1;
    }

    match set_errno(rustix::io::readv(
        &BorrowedFd::borrow_raw_fd(fd),
        slice::from_raw_parts(iov, iovcnt as usize),
    )) {
        Some(nread) => nread as isize,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn pread64(fd: c_int, ptr: *mut c_void, len: usize, offset: i64) -> isize {
    libc!(pread64(fd, ptr, len, offset));

    match set_errno(rustix::io::pread(
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
    libc!(pwrite64(fd, ptr, len, offset));

    match set_errno(rustix::io::pwrite(
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
    libc!(poll(same_ptr_mut(fds), nfds, timeout));

    let fds = slice::from_raw_parts_mut(fds, nfds.try_into().unwrap());
    match set_errno(rustix::io::poll(fds, timeout)) {
        Some(num) => num.try_into().unwrap(),
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn close(fd: c_int) -> c_int {
    libc!(close(fd));

    rustix::io::close(fd);
    0
}

#[no_mangle]
unsafe extern "C" fn dup2(fd: c_int, to: c_int) -> c_int {
    libc!(dup2(fd, to));

    let to = OwnedFd::from_raw_fd(to).into();
    match set_errno(rustix::io::dup2(&BorrowedFd::borrow_raw_fd(fd), &to)) {
        Some(()) => OwnedFd::from(to).into_raw_fd(),
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn isatty(fd: c_int) -> c_int {
    libc!(isatty(fd));

    if rustix::io::isatty(&BorrowedFd::borrow_raw_fd(fd)) {
        1
    } else {
        *__errno_location() = rustix::io::Error::NOTTY.raw_os_error();
        0
    }
}

#[no_mangle]
unsafe extern "C" fn ioctl(fd: c_int, request: c_long, mut args: ...) -> c_int {
    match request {
        data::TCGETS => {
            libc!(ioctl(fd, TCGETS));
            let fd = BorrowedFd::borrow_raw_fd(fd);
            match set_errno(rustix::io::ioctl_tcgets(&fd)) {
                Some(x) => {
                    *args.arg::<*mut rustix::io::Termios>() = x;
                    0
                }
                None => -1,
            }
        }
        data::FIONBIO => {
            libc!(ioctl(fd, FIONBIO));
            let fd = BorrowedFd::borrow_raw_fd(fd);
            let ptr = args.arg::<*mut c_int>();
            let value = *ptr != 0;
            match set_errno(rustix::io::ioctl_fionbio(&fd, value)) {
                Some(()) => 0,
                None => -1,
            }
        }
        _ => panic!("unrecognized ioctl({})", request),
    }
}

#[no_mangle]
unsafe extern "C" fn pipe2(pipefd: *mut c_int, flags: c_int) -> c_int {
    libc!(pipe2(pipefd, flags));

    let flags = PipeFlags::from_bits(flags as _).unwrap();
    match set_errno(rustix::io::pipe_with(flags)) {
        Some((a, b)) => {
            *pipefd = a.into_raw_fd();
            *pipefd.add(1) = b.into_raw_fd();
            0
        }
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn sendfile() {
    //libc!(sendfile());
    unimplemented!("sendfile")
}

#[no_mangle]
unsafe extern "C" fn sendmsg() {
    //libc!(sendmsg());
    unimplemented!("sendmsg")
}

#[no_mangle]
unsafe extern "C" fn recvmsg() {
    //libc!(recvmsg());
    unimplemented!("recvmsg")
}

#[no_mangle]
unsafe extern "C" fn epoll_create1(_flags: c_int) -> c_int {
    libc!(epoll_create1(_flags));
    unimplemented!("epoll_create1")
}

#[no_mangle]
unsafe extern "C" fn epoll_ctl(
    _epfd: c_int,
    _op: c_int,
    _fd: c_int,
    _event: *mut data::EpollEvent,
) -> c_int {
    libc!(epoll_ctl(_epfd, _op, _fd, same_ptr_mut(_event)));
    unimplemented!("epoll_ctl")
}

#[no_mangle]
unsafe extern "C" fn epoll_wait(
    _epfd: c_int,
    _events: *mut data::EpollEvent,
    _maxevents: c_int,
    _timeout: c_int,
) -> c_int {
    libc!(epoll_wait(
        _epfd,
        same_ptr_mut(_events),
        _maxevents,
        _timeout
    ));
    unimplemented!("epoll_wait")
}

#[no_mangle]
unsafe extern "C" fn eventfd(initval: c_uint, flags: c_int) -> c_int {
    libc!(eventfd(initval, flags));
    let flags = EventfdFlags::from_bits(flags.try_into().unwrap()).unwrap();
    match set_errno(rustix::io::eventfd(initval, flags)) {
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
fn tagged_alloc(type_layout: std::alloc::Layout) -> *mut u8 {
    if type_layout.size() == 0 {
        return std::ptr::null_mut();
    }

    let tag_layout = std::alloc::Layout::new::<Tag>();
    let tag = Tag {
        size: type_layout.size(),
        align: type_layout.align(),
    };
    if let Ok((total_layout, offset)) = tag_layout.extend(type_layout) {
        let total_ptr = unsafe { std::alloc::alloc(total_layout) };
        if total_ptr.is_null() {
            return total_ptr;
        }

        let tag_offset = offset - tag_layout.size();
        unsafe {
            total_ptr.wrapping_add(tag_offset).cast::<Tag>().write(tag);
            total_ptr.wrapping_add(offset).cast::<_>()
        }
    } else {
        std::ptr::null_mut()
    }
}

/// get the layout out of a tagged allocation
///
/// #Safety
/// the given pointer must be a non-null pointer,
/// gotten from calling `tagged_alloc`
unsafe fn get_layout(ptr: *mut u8) -> std::alloc::Layout {
    let tag = ptr
        .wrapping_sub(std::mem::size_of::<Tag>())
        .cast::<Tag>()
        .read();
    std::alloc::Layout::from_size_align_unchecked(tag.size, tag.align)
}

/// get the layout out of a tagged allocation
///
/// #Safety
/// the given pointer must be a non-null pointer,
/// gotten from calling `tagged_alloc`
unsafe fn tagged_dealloc(ptr: *mut u8) {
    let tag_layout = std::alloc::Layout::new::<Tag>();
    let type_layout = get_layout(ptr);
    if let Ok((total_layout, offset)) = tag_layout.extend(type_layout) {
        let total_ptr = ptr.wrapping_sub(offset);
        std::alloc::dealloc(total_ptr, total_layout);
    }
}

#[no_mangle]
unsafe extern "C" fn malloc(size: usize) -> *mut c_void {
    libc!(malloc(size));

    let layout = std::alloc::Layout::from_size_align(size, data::ALIGNOF_MAXALIGN_T).unwrap();
    tagged_alloc(layout).cast::<_>()
}

#[no_mangle]
unsafe extern "C" fn realloc(old: *mut c_void, size: usize) -> *mut c_void {
    libc!(realloc(old, size));

    if old.is_null() {
        malloc(size)
    } else {
        let old_layout = get_layout(old.cast::<_>());
        if old_layout.size() >= size {
            return old;
        }

        let new = malloc(size);
        memcpy(new, old, std::cmp::min(size, old_layout.size()));
        tagged_dealloc(old.cast::<_>());
        new
    }
}

#[no_mangle]
unsafe extern "C" fn calloc(nmemb: usize, size: usize) -> *mut c_void {
    libc!(calloc(nmemb, size));

    let product = match nmemb.checked_mul(size) {
        Some(product) => product,
        None => {
            *__errno_location() = rustix::io::Error::NOMEM.raw_os_error();
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
    libc!(posix_memalign(memptr, alignment, size));
    if !(alignment.is_power_of_two() && alignment % std::mem::size_of::<*const c_void>() == 0) {
        return rustix::io::Error::INVAL.raw_os_error();
    }

    let layout = std::alloc::Layout::from_size_align(size, alignment).unwrap();
    let ptr = tagged_alloc(layout).cast::<_>();

    *memptr = ptr;
    0
}

#[no_mangle]
unsafe extern "C" fn free(ptr: *mut c_void) {
    libc!(free(ptr));

    if ptr.is_null() {
        return;
    }

    tagged_dealloc(ptr.cast::<_>());
}

// mem

#[no_mangle]
unsafe extern "C" fn memcmp(a: *const c_void, b: *const c_void, len: usize) -> c_int {
    libc!(memcmp(a, b, len));

    compiler_builtins::mem::memcmp(a.cast::<_>(), b.cast::<_>(), len)
}

#[no_mangle]
unsafe extern "C" fn bcmp(a: *const c_void, b: *const c_void, len: usize) -> c_int {
    // `bcmp` is just an alias for `memcmp`.
    libc!(memcmp(a, b, len));

    compiler_builtins::mem::bcmp(a.cast::<_>(), b.cast::<_>(), len)
}

#[no_mangle]
unsafe extern "C" fn memcpy(dst: *mut c_void, src: *const c_void, len: usize) -> *mut c_void {
    libc!(memcpy(dst, src, len));

    compiler_builtins::mem::memcpy(dst.cast::<_>(), src.cast::<_>(), len).cast::<_>()
}

#[no_mangle]
unsafe extern "C" fn memmove(dst: *mut c_void, src: *const c_void, len: usize) -> *mut c_void {
    libc!(memmove(dst, src, len));

    compiler_builtins::mem::memmove(dst.cast::<_>(), src.cast::<_>(), len).cast::<_>()
}

#[no_mangle]
unsafe extern "C" fn memset(dst: *mut c_void, fill: c_int, len: usize) -> *mut c_void {
    libc!(memset(dst, fill, len));

    compiler_builtins::mem::memset(dst.cast::<_>(), fill, len).cast::<_>()
}

#[no_mangle]
unsafe extern "C" fn memchr(s: *const c_void, c: c_int, len: usize) -> *mut c_void {
    libc!(memchr(s, c, len));

    let slice: &[u8] = slice::from_raw_parts(s.cast(), len);
    match memchr::memchr(c as u8, slice) {
        None => null_mut(),
        Some(i) => s.cast::<u8>().add(i) as *mut c_void,
    }
}

#[no_mangle]
unsafe extern "C" fn memrchr(s: *const c_void, c: c_int, len: usize) -> *mut c_void {
    libc!(memrchr(s, c, len));

    let slice: &[u8] = slice::from_raw_parts(s.cast(), len);
    match memchr::memrchr(c as u8, slice) {
        None => null_mut(),
        Some(i) => s.cast::<u8>().add(i) as *mut c_void,
    }
}

#[no_mangle]
unsafe extern "C" fn strlen(mut s: *const c_char) -> usize {
    libc!(strlen(s));

    let mut len = 0;
    while *s != b'\0' as _ {
        len += 1;
        s = s.add(1);
    }
    len
}

// mmap

#[no_mangle]
unsafe extern "C" fn mmap(
    addr: *mut c_void,
    length: usize,
    prot: c_int,
    flags: c_int,
    fd: c_int,
    offset: c_long,
) -> *mut c_void {
    libc!(mmap(addr, length, prot, flags, fd, offset));

    let anon = flags & data::MAP_ANONYMOUS == data::MAP_ANONYMOUS;
    let prot = ProtFlags::from_bits(prot as _).unwrap();
    let flags = MapFlags::from_bits((flags & !data::MAP_ANONYMOUS) as _).unwrap();
    match set_errno(if anon {
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
        None => data::MAP_FAILED,
    }
}

#[no_mangle]
unsafe extern "C" fn mmap64(
    addr: *mut c_void,
    length: usize,
    prot: c_int,
    flags: c_int,
    fd: c_int,
    offset: i64,
) -> *mut c_void {
    libc!(mmap64(addr, length, prot, flags, fd, offset));

    let anon = flags & data::MAP_ANONYMOUS == data::MAP_ANONYMOUS;
    let prot = ProtFlags::from_bits(prot as _).unwrap();
    let flags = MapFlags::from_bits((flags & !data::MAP_ANONYMOUS) as _).unwrap();
    match set_errno(if anon {
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
        None => data::MAP_FAILED,
    }
}

#[no_mangle]
unsafe extern "C" fn munmap(ptr: *mut c_void, len: usize) -> c_int {
    libc!(munmap(ptr, len));

    match set_errno(rustix::io::munmap(ptr, len)) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn mremap(
    old_address: *mut c_void,
    old_size: usize,
    new_size: usize,
    flags: c_int,
    mut args: ...
) -> *mut c_void {
    if (flags & data::MREMAP_FIXED) == data::MREMAP_FIXED {
        let new_address = args.arg::<*mut c_void>();
        libc!(mremap(old_address, old_size, new_size, flags, new_address));

        let flags = flags & !data::MREMAP_FIXED;
        let flags = MremapFlags::from_bits(flags as _).unwrap();
        match set_errno(rustix::io::mremap_fixed(
            old_address,
            old_size,
            new_size,
            flags,
            new_address,
        )) {
            Some(new_address) => new_address,
            None => data::MAP_FAILED,
        }
    } else {
        libc!(mremap(old_address, old_size, new_size, flags));

        let flags = MremapFlags::from_bits(flags as _).unwrap();
        match set_errno(rustix::io::mremap(old_address, old_size, new_size, flags)) {
            Some(new_address) => new_address,
            None => data::MAP_FAILED,
        }
    }
}

#[no_mangle]
unsafe extern "C" fn mprotect(addr: *mut c_void, length: usize, prot: c_int) -> c_int {
    libc!(mprotect(addr, length, prot));

    let prot = MprotectFlags::from_bits(prot as _).unwrap();
    match set_errno(rustix::io::mprotect(addr, length, prot)) {
        Some(()) => 0,
        None => -1,
    }
}

// rand

#[no_mangle]
unsafe extern "C" fn getrandom(buf: *mut c_void, buflen: usize, flags: u32) -> isize {
    libc!(getrandom(buf, buflen, flags));

    if buflen == 0 {
        return 0;
    }
    let flags = rustix::rand::GetRandomFlags::from_bits(flags).unwrap();
    match set_errno(rustix::rand::getrandom(
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
    //libc!(tcgetattr());
    unimplemented!("tcgetattr")
}

#[no_mangle]
unsafe extern "C" fn tcsetattr() {
    //libc!(tcsetattr());
    unimplemented!("tcsetattr")
}

#[no_mangle]
unsafe extern "C" fn cfmakeraw() {
    //libc!(cfmakeraw());
    unimplemented!("cfmakeraw")
}

// process

#[no_mangle]
unsafe extern "C" fn chroot(_name: *const c_char) -> c_int {
    libc!(chroot(_name));
    unimplemented!("chroot")
}

#[no_mangle]
unsafe extern "C" fn sysconf(name: c_int) -> c_long {
    libc!(sysconf(name));

    match name {
        data::_SC_PAGESIZE => rustix::process::page_size() as _,
        data::_SC_GETPW_R_SIZE_MAX => -1,
        // TODO: Oddly, only ever one processor seems to be online.
        data::_SC_NPROCESSORS_ONLN => 1,
        data::_SC_SYMLOOP_MAX => data::SYMLOOP_MAX,
        _ => panic!("unrecognized sysconf({})", name),
    }
}

#[no_mangle]
unsafe extern "C" fn getcwd(buf: *mut c_char, len: usize) -> *mut c_char {
    libc!(getcwd(buf, len));

    match set_errno(rustix::process::getcwd(Vec::new())) {
        Some(path) => {
            let path = path.as_bytes();
            if path.len() + 1 <= len {
                memcpy(buf.cast(), path.as_ptr().cast(), path.len());
                *buf.add(path.len()) = 0;
                buf
            } else {
                *__errno_location() = rustix::io::Error::RANGE.raw_os_error();
                null_mut()
            }
        }
        None => null_mut(),
    }
}

#[no_mangle]
unsafe extern "C" fn chdir(path: *const c_char) -> c_int {
    libc!(chdir(path));

    let path = CStr::from_ptr(path);
    match set_errno(rustix::process::chdir(path)) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn getauxval(type_: c_ulong) -> c_ulong {
    libc!(getauxval(type_));
    unimplemented!("unrecognized getauxval {}", type_)
}

#[cfg(target_arch = "aarch64")]
#[no_mangle]
unsafe extern "C" fn __getauxval(type_: c_ulong) -> c_ulong {
    libc!(getauxval(type_));
    match type_ {
        data::AT_HWCAP => rustix::process::linux_hwcap().0 as c_ulong,
        _ => unimplemented!("unrecognized __getauxval {}", type_),
    }
}

#[no_mangle]
unsafe extern "C" fn dl_iterate_phdr(
    callback: Option<
        unsafe extern "C" fn(info: *mut data::DlPhdrInfo, size: usize, data: *mut c_void) -> c_int,
    >,
    data: *mut c_void,
) -> c_int {
    extern "C" {
        static mut __executable_start: c_void;
    }

    // Disabled for now, as our `DlPhdrInfo` has fewer fields than libc's.
    //libc!(dl_iterate_phdr(callback, data));

    let (phdr, phnum) = rustix::runtime::exe_phdrs();
    let mut info = data::DlPhdrInfo {
        dlpi_addr: &mut __executable_start as *mut _ as usize,
        dlpi_name: b"/proc/self/exe\0".as_ptr().cast(),
        dlpi_phdr: phdr.cast(),
        dlpi_phnum: phnum.try_into().unwrap(),
    };
    callback.unwrap()(&mut info, size_of::<data::DlPhdrInfo>(), data)
}

#[no_mangle]
unsafe extern "C" fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void {
    libc!(dlsym(handle, symbol));

    if handle.is_null() {
        // `std` uses `dlsym` to dynamically detect feature availability; recognize
        // functions it asks for.
        if CStr::from_ptr(symbol).to_bytes() == b"statx" {
            return statx as *mut c_void;
        }
        if CStr::from_ptr(symbol).to_bytes() == b"getrandom" {
            return getrandom as *mut c_void;
        }
        if CStr::from_ptr(symbol).to_bytes() == b"copy_file_range" {
            return copy_file_range as *mut c_void;
        }
        if CStr::from_ptr(symbol).to_bytes() == b"clone3" {
            // Let's just say we don't support this for now.
            return null_mut();
        }
        if CStr::from_ptr(symbol).to_bytes() == b"__pthread_get_minstack" {
            // Let's just say we don't support this for now.
            return null_mut();
        }
        if CStr::from_ptr(symbol).to_bytes() == b"gnu_get_libc_version" {
            return gnu_get_libc_version as *mut c_void;
        }
    }
    unimplemented!("dlsym({:?})", CStr::from_ptr(symbol))
}

#[no_mangle]
unsafe extern "C" fn dlclose() {
    //libc!(dlclose());
    unimplemented!("dlclose")
}

#[no_mangle]
unsafe extern "C" fn dlerror() {
    //libc!(dlerror());
    unimplemented!("dlerror")
}

#[no_mangle]
unsafe extern "C" fn dlopen() {
    //libc!(dlopen());
    unimplemented!("dlopen")
}

#[no_mangle]
unsafe extern "C" fn __tls_get_addr(p: &[usize; 2]) -> *mut c_void {
    //libc!(__tls_get_addr(p));
    let [module, offset] = *p;
    // Offset 0 is the generation field, and we don't support dynamic linking,
    // so we should only sever see 1 here.
    assert_eq!(module, 1);
    origin::current_thread_tls_addr(offset)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "C" fn ___tls_get_addr() {
    //libc!(___tls_get_addr());
    unimplemented!("___tls_get_addr")
}

#[no_mangle]
unsafe extern "C" fn sched_yield() -> c_int {
    libc!(sched_yield());

    rustix::process::sched_yield();
    0
}

fn set_cpu(cpu: usize, cpuset: &mut data::CpuSet) {
    let size_in_bits = 8 * std::mem::size_of_val(&cpuset.bits[0]); // 32, 64 etc
    let (idx, offset) = (cpu / size_in_bits, cpu % size_in_bits);
    cpuset.bits[idx] |= 1 << offset;
}

#[no_mangle]
unsafe extern "C" fn sched_getaffinity(
    pid: c_int,
    cpu_set_size: usize,
    mask: *mut data::CpuSet,
) -> c_int {
    libc!(sched_getaffinity(pid, cpu_set_size, mask.cast::<_>()));
    let pid = rustix::process::Pid::from_raw(pid as _);
    match set_errno(rustix::process::sched_getaffinity(pid)) {
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

#[no_mangle]
unsafe extern "C" fn prctl(
    option: c_int,
    arg2: c_ulong,
    _arg3: c_ulong,
    _arg4: c_ulong,
    _arg5: c_ulong,
) -> c_int {
    libc!(prctl(option, arg2, _arg3, _arg4, _arg5));
    match option {
        data::PR_SET_NAME => {
            match set_errno(rustix::runtime::set_thread_name(CStr::from_ptr(
                arg2 as *const c_char,
            ))) {
                Some(()) => 0,
                None => -1,
            }
        }
        _ => unimplemented!("unrecognized prctl op {}", option),
    }
}

#[no_mangle]
unsafe extern "C" fn setgid() {
    //libc!(setgid());
    unimplemented!("setgid")
}

#[no_mangle]
unsafe extern "C" fn setgroups() {
    //libc!(setgroups());
    unimplemented!("setgroups")
}

#[no_mangle]
unsafe extern "C" fn setuid() {
    //libc!(setuid());
    unimplemented!("setuid")
}

#[no_mangle]
unsafe extern "C" fn getpid() -> c_int {
    libc!(getpid());
    rustix::process::getpid().as_raw() as _
}

#[no_mangle]
unsafe extern "C" fn getppid() -> c_int {
    libc!(getppid());
    rustix::process::getppid().as_raw() as _
}

#[no_mangle]
unsafe extern "C" fn getuid() -> c_uint {
    libc!(getuid());
    rustix::process::getuid().as_raw()
}

#[no_mangle]
unsafe extern "C" fn getgid() -> c_uint {
    libc!(getgid());
    rustix::process::getgid().as_raw()
}

#[no_mangle]
unsafe extern "C" fn kill(_pid: c_int, _sig: c_int) -> c_int {
    libc!(kill(_pid, _sig));
    unimplemented!("kill")
}

#[no_mangle]
unsafe extern "C" fn raise(_sig: c_int) -> c_int {
    libc!(raise(_sig));
    unimplemented!("raise")
}

// nss

#[no_mangle]
unsafe extern "C" fn getpwuid_r() {
    //libc!(getpwuid_r());
    unimplemented!("getpwuid_r")
}

// signal

#[no_mangle]
unsafe extern "C" fn abort() {
    libc!(abort());
    unimplemented!("abort")
}

#[no_mangle]
unsafe extern "C" fn signal(_num: c_int, _handler: usize) -> usize {
    libc!(signal(_num, _handler));

    // Somehow no signals for this handler were ever delivered. How odd.
    data::SIG_DFL
}

#[no_mangle]
unsafe extern "C" fn sigaction(_signum: c_int, _act: *const c_void, _oldact: *mut c_void) -> c_int {
    libc!(sigaction(_signum, _act.cast::<_>(), _oldact.cast::<_>()));

    // No signals for this handler were ever delivered either. What a coincidence.
    0
}

#[no_mangle]
unsafe extern "C" fn sigaltstack(_ss: *const c_void, _old_ss: *mut c_void) -> c_int {
    libc!(sigaltstack(_ss.cast::<_>(), _old_ss.cast::<_>()));

    // Somehow no signals were delivered and the stack wasn't ever used. What luck.
    0
}

#[no_mangle]
unsafe extern "C" fn sigaddset(_sigset: *mut c_void, _signum: c_int) -> c_int {
    libc!(sigaddset(_sigset.cast::<_>(), _signum));
    // Signal sets are not used right now.
    0
}

#[no_mangle]
unsafe extern "C" fn sigemptyset(_sigset: *mut c_void) -> c_int {
    libc!(sigemptyset(_sigset.cast::<_>()));
    // Signal sets are not used right now.
    0
}

// syscall

#[no_mangle]
unsafe extern "C" fn syscall(number: c_long, mut args: ...) -> c_long {
    match number {
        data::SYS_getrandom => {
            let buf = args.arg::<*mut c_void>();
            let len = args.arg::<usize>();
            let flags = args.arg::<u32>();
            libc!(syscall(SYS_getrandom, buf, len, flags));
            getrandom(buf, len, flags) as _
        }
        data::SYS_futex => {
            use rustix::thread::{futex, FutexFlags, FutexOperation};

            let uaddr = args.arg::<*mut u32>();
            let futex_op = args.arg::<c_int>();
            let val = args.arg::<u32>();
            let timeout = args.arg::<*const data::OldTimespec>();
            let uaddr2 = args.arg::<*mut u32>();
            let val3 = args.arg::<u32>();
            libc!(syscall(
                SYS_futex, uaddr, futex_op, val, timeout, uaddr2, val3
            ));
            let flags = FutexFlags::from_bits_truncate(futex_op as _);
            let op = match futex_op & (!flags.bits() as i32) {
                data::FUTEX_WAIT => FutexOperation::Wait,
                data::FUTEX_WAKE => FutexOperation::Wake,
                data::FUTEX_FD => FutexOperation::Fd,
                data::FUTEX_REQUEUE => FutexOperation::Requeue,
                data::FUTEX_CMP_REQUEUE => FutexOperation::CmpRequeue,
                data::FUTEX_WAKE_OP => FutexOperation::WakeOp,
                data::FUTEX_LOCK_PI => FutexOperation::LockPi,
                data::FUTEX_UNLOCK_PI => FutexOperation::UnlockPi,
                data::FUTEX_TRYLOCK_PI => FutexOperation::TrylockPi,
                data::FUTEX_WAIT_BITSET => FutexOperation::WaitBitset,
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
            match set_errno(futex(uaddr, op, flags, val, new_timespec, uaddr2, val3)) {
                Some(result) => result as _,
                None => -1,
            }
        }
        data::SYS_clone3 => {
            // ensure std::process uses fork as fallback code on linux
            set_errno::<()>(Err(rustix::io::Error::NOSYS));
            -1
        }
        _ => unimplemented!("syscall({:?})", number),
    }
}

// posix_spawn

#[no_mangle]
unsafe extern "C" fn posix_spawnp() {
    //libc!(posix_spawnp());
    unimplemented!("posix_spawnp");
}

#[no_mangle]
unsafe extern "C" fn posix_spawnattr_destroy(ptr: *const c_void) -> c_int {
    //libc!(posix_spawn_spawnattr_destroy(ptr));
    rustix::io::write(
        &rustix::io::stderr(),
        b"unimplemented: posix_spawn_spawnattr_destroy\n",
    )
    .ok();
    0
}

#[no_mangle]
unsafe extern "C" fn posix_spawnattr_init(ptr: *const c_void) -> c_int {
    //libc!(posix_spawnattr_init(ptr));
    rustix::io::write(
        &rustix::io::stderr(),
        b"unimplemented: posix_spawnattr_init\n",
    )
    .ok();
    0
}

#[no_mangle]
unsafe extern "C" fn posix_spawnattr_setflags() {
    //libc!(posix_spawnattr_setflags());
    unimplemented!("posix_spawnattr_setflags")
}

#[no_mangle]
unsafe extern "C" fn posix_spawnattr_setsigdefault() {
    //libc!(posix_spawnattr_setsigdefault());
    unimplemented!("posix_spawnattr_setsigdefault")
}

#[no_mangle]
unsafe extern "C" fn posix_spawnattr_setsigmask() {
    //libc!(posix_spawnattr_setsigmask());
    unimplemented!("posix_spawnsetsigmask")
}

#[no_mangle]
unsafe extern "C" fn posix_spawn_file_actions_adddup2() {
    //libc!(posix_spawn_file_actions_adddup2());
    unimplemented!("posix_spawn_file_actions_adddup2")
}

#[no_mangle]
unsafe extern "C" fn posix_spawn_file_actions_destroy(ptr: *const c_void) -> c_int {
    //libc!(posix_spawn_file_actions_destroy(ptr));
    rustix::io::write(
        &rustix::io::stderr(),
        b"unimplemented: posix_spawn_file_actions_destroy\n",
    )
    .ok();
    0
}

#[no_mangle]
unsafe extern "C" fn posix_spawn_file_actions_init(ptr: *const c_void) -> c_int {
    //libc!(posix_spawn_file_actions_init(ptr));
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
    libc!(clock_gettime(id, same_ptr_mut(tp)));

    let id = match id {
        data::CLOCK_MONOTONIC => rustix::time::ClockId::Monotonic,
        data::CLOCK_REALTIME => rustix::time::ClockId::Realtime,
        _ => panic!("unimplemented clock({})", id),
    };
    *tp = rustix::time::clock_gettime(id);
    0
}

#[no_mangle]
unsafe extern "C" fn nanosleep(
    req: *const data::OldTimespec,
    rem: *mut data::OldTimespec,
) -> c_int {
    libc!(nanosleep(same_ptr(req), same_ptr_mut(rem)));

    let req = rustix::time::Timespec {
        tv_sec: (*req).tv_sec.into(),
        tv_nsec: (*req).tv_nsec as _,
    };
    match rustix::time::nanosleep(&req) {
        rustix::time::NanosleepRelativeResult::Ok => 0,
        rustix::time::NanosleepRelativeResult::Interrupted(remaining) => {
            *rem = data::OldTimespec {
                tv_sec: remaining.tv_sec.try_into().unwrap(),
                tv_nsec: remaining.tv_nsec as _,
            };
            *__errno_location() = rustix::io::Error::INTR.raw_os_error();
            -1
        }
        rustix::time::NanosleepRelativeResult::Err(err) => {
            *__errno_location() = err.raw_os_error();
            -1
        }
    }
}

// exec

unsafe fn null_terminated_array<'a>(list: *const *const c_char) -> Vec<&'a CStr> {
    let mut len = 0;
    while !(*list.wrapping_add(len)).is_null() {
        len += 1;
    }
    let list = std::slice::from_raw_parts(list, len);
    list.into_iter()
        .copied()
        .map(|ptr| CStr::from_ptr(ptr))
        .collect()
}

fn file_exists(cwd: &rustix::io_lifetimes::BorrowedFd<'_>, path: &CStr) -> bool {
    rustix::fs::accessat(
        cwd,
        path,
        rustix::fs::Access::EXISTS | rustix::fs::Access::EXEC_OK,
        rustix::fs::AtFlags::empty(),
    )
    .is_ok()
}

fn resolve_binary<'a>(file: &'a CStr, envs: &[&CStr]) -> rustix::io::Result<Cow<'a, CStr>> {
    let file_bytes = file.to_bytes();
    if file_bytes.contains(&b'/') {
        Ok(Cow::Borrowed(file))
    } else {
        let cwd = rustix::fs::cwd();
        match envs
            .into_iter()
            .copied()
            .map(CStr::to_bytes)
            .find(|env| env.starts_with(b"PATH="))
            .map(|env| env.split_at(b"PATH=".len()).1)
            .unwrap_or(b"/bin:/usr/bin")
            .split(|&byte| byte == b':')
            .filter_map(|path_bytes| {
                let mut full_path = path_bytes.to_vec();
                full_path.push(b'/');
                full_path.extend_from_slice(file_bytes);
                CString::new(full_path).ok()
            })
            .find(|path| file_exists(&cwd, &path))
        {
            Some(path) => Ok(Cow::Owned(path)),
            None => Err(rustix::io::Error::ACCES),
        }
    }
}

#[no_mangle]
unsafe extern "C" fn execvp(file: *const c_char, args: *const *const c_char) -> c_int {
    libc!(execvp(file, args));
    let args = null_terminated_array(args);
    let envs = null_terminated_array(environ.cast::<_>());
    match set_errno(
        resolve_binary(CStr::from_ptr(file), &envs)
            .and_then(|path| rustix::runtime::execve(path, &args, &envs)),
    ) {
        Some(_) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn fork() -> c_int {
    libc!(fork());
    match set_errno(origin::fork()) {
        Some(Some(pid)) => pid.as_raw() as c_int,
        Some(None) => 0,
        None => -1,
    }
}

// <https://refspecs.linuxbase.org/LSB_5.0.0/LSB-Core-generic/LSB-Core-generic/baselib---register-atfork.html>
#[no_mangle]
unsafe extern "C" fn __register_atfork(
    prepare: Option<unsafe extern "C" fn()>,
    parent: Option<unsafe extern "C" fn()>,
    child: Option<unsafe extern "C" fn()>,
    _dso_handle: *mut c_void,
) -> c_int {
    libc!(__register_atfork(prepare, parent, child, _dso_handle));
    origin::at_fork(prepare, parent, child);
    0
}

#[no_mangle]
unsafe extern "C" fn clone3() {
    //libc!(clone3());
    unimplemented!("clone3")
}

#[no_mangle]
unsafe extern "C" fn waitpid(pid: c_int, status: *mut c_int, options: c_int) -> c_int {
    libc!(waitpid(pid, status, options));
    let options = WaitOptions::from_bits(options as _).unwrap();
    let ret_pid;
    let ret_status;
    match pid {
        -1 => match set_errno(rustix::process::wait(options)) {
            Some(Some((new_pid, new_status))) => {
                ret_pid = new_pid.as_raw() as c_int;
                ret_status = new_status.as_raw() as c_int;
            }
            Some(None) => return 0,
            None => return -1,
        },
        pid => match set_errno(rustix::process::waitpid(
            rustix::process::Pid::from_raw(pid as _),
            options,
        )) {
            Some(Some(new_status)) => {
                ret_pid = if pid == 0 {
                    rustix::process::getpid().as_raw() as c_int
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
    //libc!(siglongjmp());
    unimplemented!("siglongjmp")
}

#[no_mangle]
unsafe extern "C" fn __sigsetjmp() {
    //libc!(__sigsetjmp());
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
    libc!(getenv(key));
    let key = CStr::from_ptr(key);
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
unsafe extern "C" fn setenv() {
    //libc!(setenv());
    unimplemented!("setenv")
}

#[no_mangle]
unsafe extern "C" fn unsetenv() {
    //libc!(unsetenv());
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

    fn nonzero_thread_id(&self) -> std::num::NonZeroUsize {
        (origin::current_thread_id().as_raw() as usize)
            .try_into()
            .unwrap()
    }
}

#[cfg(feature = "threads")]
#[allow(non_camel_case_types)]
type PthreadT = c_ulong;
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
    libc!(pthread_self());
    origin::current_thread() as PthreadT
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_getattr_np(thread: PthreadT, attr: *mut PthreadAttrT) -> c_int {
    libc!(pthread_getattr_np(thread, same_ptr_mut(attr)));
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
    libc!(pthread_attr_init(same_ptr_mut(attr)));
    ptr::write(attr, PthreadAttrT::default());
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_attr_destroy(attr: *mut PthreadAttrT) -> c_int {
    libc!(pthread_attr_destroy(same_ptr_mut(attr)));
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
    libc!(pthread_attr_getstack(same_ptr(attr), stackaddr, stacksize));
    *stackaddr = (*attr).stack_addr;
    *stacksize = (*attr).stack_size;
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_getspecific() -> *const c_void {
    //libc!(pthread_getspecific());
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
    //libc!(pthread_key_create());
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
    //libc!(pthread_key_delete());
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
    libc!(pthread_mutexattr_destroy(same_ptr_mut(attr)));
    ptr::drop_in_place(attr);
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_mutexattr_init(attr: *mut PthreadMutexattrT) -> c_int {
    libc!(pthread_mutexattr_init(same_ptr_mut(attr)));
    ptr::write(
        attr,
        PthreadMutexattrT {
            kind: AtomicU32::new(data::PTHREAD_MUTEX_DEFAULT as u32),
        },
    );
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_mutexattr_settype(attr: *mut PthreadMutexattrT, kind: c_int) -> c_int {
    libc!(pthread_mutexattr_settype(same_ptr_mut(attr), kind));
    (*attr).kind = AtomicU32::new(kind as u32);
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_mutex_destroy(mutex: *mut PthreadMutexT) -> c_int {
    libc!(pthread_mutex_destroy(same_ptr_mut(mutex)));
    match (*mutex).kind.load(SeqCst) as i32 {
        data::PTHREAD_MUTEX_NORMAL => ptr::drop_in_place(&mut (*mutex).u.normal),
        data::PTHREAD_MUTEX_RECURSIVE => ptr::drop_in_place(&mut (*mutex).u.reentrant),
        data::PTHREAD_MUTEX_ERRORCHECK => unimplemented!("PTHREAD_MUTEX_ERRORCHECK"),
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
    libc!(pthread_mutex_init(same_ptr_mut(mutex), same_ptr(mutexattr)));
    let kind = (*mutexattr).kind.load(SeqCst);
    match kind as i32 {
        data::PTHREAD_MUTEX_NORMAL => ptr::write(&mut (*mutex).u.normal, RawMutex::INIT),
        data::PTHREAD_MUTEX_RECURSIVE => ptr::write(
            &mut (*mutex).u.reentrant,
            parking_lot::lock_api::RawReentrantMutex::<parking_lot::RawMutex, GetThreadId>::INIT,
        ),
        data::PTHREAD_MUTEX_ERRORCHECK => unimplemented!("PTHREAD_MUTEX_ERRORCHECK"),
        other => unimplemented!("unsupported pthread mutex kind {}", other),
    }
    (*mutex).kind.store(kind, SeqCst);
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_mutex_lock(mutex: *mut PthreadMutexT) -> c_int {
    libc!(pthread_mutex_lock(same_ptr_mut(mutex)));
    match (*mutex).kind.load(SeqCst) as i32 {
        data::PTHREAD_MUTEX_NORMAL => (*mutex).u.normal.lock(),
        data::PTHREAD_MUTEX_RECURSIVE => (*mutex).u.reentrant.lock(),
        data::PTHREAD_MUTEX_ERRORCHECK => unimplemented!("PTHREAD_MUTEX_ERRORCHECK"),
        other => unimplemented!("unsupported pthread mutex kind {}", other),
    }
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_mutex_trylock(mutex: *mut PthreadMutexT) -> c_int {
    libc!(pthread_mutex_trylock(same_ptr_mut(mutex)));
    if match (*mutex).kind.load(SeqCst) as i32 {
        data::PTHREAD_MUTEX_NORMAL => (*mutex).u.normal.try_lock(),
        data::PTHREAD_MUTEX_RECURSIVE => (*mutex).u.reentrant.try_lock(),
        data::PTHREAD_MUTEX_ERRORCHECK => unimplemented!("PTHREAD_MUTEX_ERRORCHECK"),
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
    libc!(pthread_mutex_unlock(same_ptr_mut(mutex)));
    match (*mutex).kind.load(SeqCst) as i32 {
        data::PTHREAD_MUTEX_NORMAL => (*mutex).u.normal.unlock(),
        data::PTHREAD_MUTEX_RECURSIVE => (*mutex).u.reentrant.unlock(),
        data::PTHREAD_MUTEX_ERRORCHECK => unimplemented!("PTHREAD_MUTEX_ERRORCHECK"),
        other => unimplemented!("unsupported pthread mutex kind {}", other),
    }
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_rwlock_tryrdlock(rwlock: *mut PthreadRwlockT) -> c_int {
    libc!(pthread_rwlock_tryrdlock(same_ptr_mut(rwlock)));
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
    libc!(pthread_rwlock_trywrlock(same_ptr_mut(rwlock)));
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
    libc!(pthread_rwlock_rdlock(same_ptr_mut(rwlock)));
    (*rwlock).lock.lock_shared();
    (*rwlock).exclusive.store(false, SeqCst);
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_rwlock_unlock(rwlock: *mut PthreadRwlockT) -> c_int {
    libc!(pthread_rwlock_unlock(same_ptr_mut(rwlock)));
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
    //libc!(pthread_setspecific());
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
    libc!(pthread_attr_getguardsize(same_ptr(attr), guardsize));
    *guardsize = (*attr).guard_size;
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_attr_setguardsize(attr: *mut PthreadAttrT, guardsize: usize) -> c_int {
    // TODO: libc!(pthread_attr_setguardsize(attr, guardsize));
    (*attr).guard_size = guardsize;
    0
}

const SIZEOF_PTHREAD_COND_T: usize = 48;

#[cfg(feature = "threads")]
#[cfg_attr(target_arch = "x86", repr(C, align(4)))]
#[cfg_attr(not(target_arch = "x86"), repr(C, align(8)))]
struct PthreadCondT {
    inner: parking_lot::Condvar,
    pad: [u8; SIZEOF_PTHREAD_COND_T - std::mem::size_of::<parking_lot::Condvar>()],
}

#[cfg(feature = "threads")]
libc_type!(PthreadCondT, pthread_cond_t);

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
    libc!(pthread_condattr_destroy(same_ptr_mut(attr)));
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
    libc!(pthread_condattr_init(same_ptr_mut(attr)));
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
    libc!(pthread_condattr_setclock(same_ptr_mut(attr), clock_id));
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
    libc!(pthread_cond_broadcast(same_ptr_mut(cond)));
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
    libc!(pthread_cond_destroy(same_ptr_mut(cond)));
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
    libc!(pthread_cond_init(same_ptr_mut(cond), same_ptr(attr)));
    let _ = (cond, attr);
    rustix::io::write(&rustix::io::stderr(), b"unimplemented: pthread_cond_init\n").ok();
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_cond_signal(cond: *mut PthreadCondT) -> c_int {
    libc!(pthread_cond_signal(same_ptr_mut(cond)));
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
    libc!(pthread_cond_wait(same_ptr_mut(cond), same_ptr_mut(lock)));
    let _ = (cond, lock);
    rustix::io::write(&rustix::io::stderr(), b"unimplemented: pthread_cond_wait\n").ok();
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_cond_timedwait(
    cond: *mut PthreadCondT,
    lock: *mut PthreadMutexT,
    abstime: *const data::OldTimespec,
) -> c_int {
    libc!(pthread_cond_timedwait(
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
    libc!(pthread_rwlock_destroy(same_ptr_mut(rwlock)));
    ptr::drop_in_place(rwlock);
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_rwlock_wrlock(rwlock: *mut PthreadRwlockT) -> c_int {
    libc!(pthread_rwlock_wrlock(same_ptr_mut(rwlock)));
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
    libc!(pthread_create(pthread, same_ptr(attr), fn_, arg));
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
    libc!(pthread_detach(pthread));
    origin::detach_thread(pthread as *mut Thread);
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_join(pthread: PthreadT, retval: *mut *mut c_void) -> c_int {
    libc!(pthread_join(pthread, retval));
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
    //libc!(pthread_sigmask());
    // not yet implemented, what is needed fo `std::Command` to work.
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_attr_setstacksize(attr: *mut PthreadAttrT, stacksize: usize) -> c_int {
    libc!(pthread_attr_setstacksize(same_ptr_mut(attr), stacksize));
    (*attr).stack_size = stacksize;
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_cancel() -> c_int {
    //libc!(pthread_cancel());
    unimplemented!("pthread_cancel")
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_exit() -> c_int {
    //libc!(pthread_exit());
    // As with `pthread_cancel`, `pthread_exit` may be tricky to implement.
    unimplemented!("pthread_exit")
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_cleanup_push() -> c_int {
    //libc!(pthread_cleanup_push());
    unimplemented!("pthread_cleanup_push")
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_cleanup_pop() -> c_int {
    //libc!(pthread_cleanup_pop());
    unimplemented!("pthread_cleanup_pop")
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_setcancelstate() -> c_int {
    //libc!(pthread_setcancelstate());
    unimplemented!("pthread_setcancelstate")
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_setcanceltype() -> c_int {
    //libc!(pthread_setcanceltype());
    unimplemented!("pthread_setcanceltype")
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_testcancel() -> c_int {
    //libc!(pthread_testcancel());
    unimplemented!("pthread_testcancel")
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn pthread_atfork(
    prepare: Option<unsafe extern "C" fn()>,
    parent: Option<unsafe extern "C" fn()>,
    child: Option<unsafe extern "C" fn()>,
) -> c_int {
    libc!(pthread_atfork(prepare, parent, child));
    origin::at_fork(prepare, parent, child);
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn __cxa_thread_atexit_impl(
    func: unsafe extern "C" fn(*mut c_void),
    obj: *mut c_void,
    _dso_symbol: *mut c_void,
) -> c_int {
    // TODO: libc!(__cxa_thread_atexit_impl(func, obj, _dso_symbol));
    let obj = obj as usize;
    origin::at_thread_exit(Box::new(move || func(obj as *mut c_void)));
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
    libc!(atexit(func));
    origin::at_exit(Box::new(move || func()));
    0
}

/// C-compatible `exit`.
///
/// Call all the registered at-exit functions, and exit the program.
#[no_mangle]
unsafe extern "C" fn exit(status: c_int) -> ! {
    libc!(exit(status));
    origin::exit(status)
}

/// POSIX-compatible `_exit`.
///
/// Just exit the process.
#[no_mangle]
unsafe extern "C" fn _exit(status: c_int) -> ! {
    libc!(_exit(status));
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

// compiler-builtins

#[cfg(target_arch = "arm")]
#[no_mangle]
unsafe extern "C" fn __aeabi_d2f(_a: f64) -> f32 {
    //libc!(__aeabi_d2f(_a));
    unimplemented!("__aeabi_d2f")
}

// utilities

fn set_errno<T>(result: Result<T, rustix::io::Error>) -> Option<T> {
    result
        .map_err(|err| unsafe {
            *__errno_location() = err.raw_os_error();
        })
        .ok()
}
