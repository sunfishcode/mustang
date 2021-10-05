#![doc = include_str!("../README.md")]
#![no_builtins] // don't let LLVM optimize our `memcpy` into a `memcpy` call
#![feature(thread_local)] // for `__errno_location`
#![feature(c_variadic)] // for `ioctl` etc.
#![feature(rustc_private)] // for compiler-builtins
#![feature(untagged_unions)] // for `PthreadMutexT`
#![feature(atomic_mut_ptr)] // for `RawMutex`
#![cfg(target_vendor = "mustang")]

#[macro_use]
mod use_libc;

mod data;
mod error_str;
#[cfg(feature = "threads")]
mod raw_mutex;
// Unwind isn't supported on x86 yet.
#[cfg(target_arch = "x86")]
mod unwind;

/// Ensure that `mustang`'s modules are linked in.
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
#[cold]
unsafe extern "C" fn __mustang_c_scape() {
    // Unwind isn't supported on x86 yet.
    #[cfg(target_arch = "x86")]
    __mustang_c_scape__unwind()
}

// Selected libc-compatible interfaces.
//
// The goal here isn't necessarily to build a complete libc; it's primarily
// to provide things that `std` and possibly popular crates are currently
// using.
//
// This effectively undoes the work that `rsix` does: it calls `rsix` and
// translates it back into a C-like ABI. Ideally, Rust code should just call
// the `rsix` APIs directly, which are safer, more ergonomic, and skip this
// whole layer.
//
// Everything here is defined in the same file, using `link_section`, to
// ensure that it's all linked in together, to ensure that we fully replace
// libc, libm, and libpthread.

use error_str::error_str;
use memoffset::offset_of;
#[cfg(feature = "threads")]
use origin::Thread;
#[cfg(feature = "threads")]
use parking_lot::lock_api::RawRwLock;
#[cfg(feature = "threads")]
use raw_mutex::RawMutex;
use rsix::fs::{cwd, openat, AtFlags, FdFlags, Mode, OFlags};
use rsix::io::{MapFlags, MprotectFlags, MremapFlags, PipeFlags, ProtFlags};
use rsix::io_lifetimes::{AsFd, BorrowedFd, OwnedFd};
use rsix::net::{
    AcceptFlags, AddressFamily, IpAddr, Ipv4Addr, Ipv6Addr, Protocol, RecvFlags, SendFlags,
    Shutdown, SocketAddr, SocketAddrStorage, SocketAddrV4, SocketAddrV6, SocketFlags, SocketType,
};
use std::convert::TryInto;
use std::ffi::{c_void, CStr, OsStr, OsString};
use std::mem::transmute;
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

#[inline(never)]
#[link_section = ".text.__mustang"]
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
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn __xpg_strerror_r(errnum: c_int, buf: *mut c_char, buflen: usize) -> c_int {
    libc!(strerror_r(errnum, buf, buflen));

    let message = match error_str(rsix::io::Error::from_raw_os_error(errnum)) {
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

#[inline(never)]
#[link_section = ".text.__mustang"]
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

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn open() {
    //libc!(open())
    unimplemented!("open")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn readlink(pathname: *const c_char, buf: *mut c_char, bufsiz: usize) -> isize {
    libc!(readlink(pathname, buf, bufsiz));

    let path = match set_errno(rsix::fs::readlinkat(
        &cwd(),
        CStr::from_ptr(pathname),
        OsString::new(),
    )) {
        Some(path) => path,
        None => return -1,
    };
    let bytes = path.as_bytes();
    let min = std::cmp::min(bytes.len(), bufsiz);
    slice::from_raw_parts_mut(buf.cast::<_>(), min).copy_from_slice(bytes);
    min as isize
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn stat() -> c_int {
    //libc!(stat())
    unimplemented!("stat")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn stat64(pathname: *const c_char, stat_: *mut rsix::fs::Stat) -> c_int {
    libc!(stat64(pathname, same_ptr_mut(stat_)));

    match set_errno(rsix::fs::statat(
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

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn fstat(_fd: c_int, _stat: *mut rsix::fs::Stat) -> c_int {
    libc!(fstat(_fd, _stat));
    unimplemented!("fstat")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn fstat64(fd: c_int, stat_: *mut rsix::fs::Stat) -> c_int {
    libc!(fstat64(fd, same_ptr_mut(stat_)));

    match set_errno(rsix::fs::fstat(&BorrowedFd::borrow_raw_fd(fd))) {
        Some(r) => {
            *stat_ = r;
            0
        }
        None => -1,
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn statx(
    dirfd_: c_int,
    path: *const c_char,
    flags: c_int,
    mask: c_uint,
    stat_: *mut rsix::fs::Statx,
) -> c_int {
    libc!(statx(dirfd_, path, flags, mask, same_ptr_mut(stat_)));

    if path.is_null() || stat_.is_null() {
        *__errno_location() = rsix::io::Error::FAULT.raw_os_error();
        return -1;
    }

    let flags = AtFlags::from_bits(flags as _).unwrap();
    let mask = rsix::fs::StatxFlags::from_bits(mask).unwrap();
    match set_errno(rsix::fs::statx(
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

#[inline(never)]
#[link_section = ".text.__mustang"]
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
                    *__errno_location() = rsix::io::Error::NOMEM.raw_os_error();
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

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn fcntl(fd: c_int, cmd: c_int, mut args: ...) -> c_int {
    match cmd {
        data::F_GETFL => {
            libc!(fcntl(fd, F_GETFL));
            let fd = BorrowedFd::borrow_raw_fd(fd);
            match set_errno(rsix::fs::fcntl_getfl(&fd)) {
                Some(flags) => flags.bits() as _,
                None => -1,
            }
        }
        data::F_SETFD => {
            let flags = args.arg::<c_int>();
            libc!(fcntl(fd, F_SETFD, flags));
            let fd = BorrowedFd::borrow_raw_fd(fd);
            match set_errno(rsix::fs::fcntl_setfd(
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
            match set_errno(rsix::fs::fcntl_dupfd_cloexec(&fd, arg)) {
                Some(fd) => fd.into_raw_fd(),
                None => -1,
            }
        }
        _ => panic!("unrecognized fnctl({})", cmd),
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn mkdir(pathname: *const c_char, mode: c_uint) -> c_int {
    libc!(mkdir(pathname, mode));

    let mode = Mode::from_bits(mode as _).unwrap();
    match set_errno(rsix::fs::mkdirat(&cwd(), CStr::from_ptr(pathname), mode)) {
        Some(()) => 0,
        None => -1,
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn fdatasync(fd: c_int) -> c_int {
    libc!(fdatasync(fd));

    match set_errno(rsix::fs::fdatasync(&BorrowedFd::borrow_raw_fd(fd))) {
        Some(()) => 0,
        None => -1,
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn fstatat64(
    fd: c_int,
    pathname: *const c_char,
    stat_: *mut rsix::fs::Stat,
    flags: c_int,
) -> c_int {
    libc!(fstatat64(fd, pathname, same_ptr_mut(stat_), flags));

    let flags = AtFlags::from_bits(flags as _).unwrap();
    match set_errno(rsix::fs::statat(
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

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn fsync(fd: c_int) -> c_int {
    libc!(fsync(fd));

    match set_errno(rsix::fs::fdatasync(&BorrowedFd::borrow_raw_fd(fd))) {
        Some(()) => 0,
        None => -1,
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn ftruncate64(fd: c_int, length: i64) -> c_int {
    libc!(ftruncate64(fd, length));

    match set_errno(rsix::fs::ftruncate(
        &BorrowedFd::borrow_raw_fd(fd),
        length as u64,
    )) {
        Some(()) => 0,
        None => -1,
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn rename(old: *const c_char, new: *const c_char) -> c_int {
    libc!(rename(old, new));

    match set_errno(rsix::fs::renameat(
        &cwd(),
        CStr::from_ptr(old),
        &cwd(),
        CStr::from_ptr(new),
    )) {
        Some(()) => 0,
        None => -1,
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn rmdir(pathname: *const c_char) -> c_int {
    libc!(rmdir(pathname));

    match set_errno(rsix::fs::unlinkat(
        &cwd(),
        CStr::from_ptr(pathname),
        AtFlags::REMOVEDIR,
    )) {
        Some(()) => 0,
        None => -1,
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn unlink(pathname: *const c_char) -> c_int {
    libc!(unlink(pathname));

    match set_errno(rsix::fs::unlinkat(
        &cwd(),
        CStr::from_ptr(pathname),
        AtFlags::empty(),
    )) {
        Some(()) => 0,
        None => -1,
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn lseek64(fd: c_int, offset: i64, whence: c_int) -> i64 {
    libc!(lseek64(fd, offset, whence));

    let seek_from = match whence {
        data::SEEK_SET => std::io::SeekFrom::Start(offset as u64),
        data::SEEK_CUR => std::io::SeekFrom::Current(offset),
        data::SEEK_END => std::io::SeekFrom::End(offset),
        _ => panic!("unrecognized whence({})", whence),
    };
    match set_errno(rsix::fs::seek(&BorrowedFd::borrow_raw_fd(fd), seek_from)) {
        Some(offset) => offset as i64,
        None => -1,
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn lstat64(pathname: *const c_char, stat_: *mut rsix::fs::Stat) -> c_int {
    libc!(lstat64(pathname, same_ptr_mut(stat_)));

    match set_errno(rsix::fs::statat(
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

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn opendir(pathname: *const c_char) -> *mut c_void {
    libc!(opendir(pathname).cast::<_>());

    match set_errno(rsix::fs::openat(
        &cwd(),
        CStr::from_ptr(pathname),
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
        Mode::empty(),
    )) {
        Some(fd) => fdopendir(fd.into_raw_fd()),
        None => null_mut(),
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn fdopendir(fd: c_int) -> *mut c_void {
    libc!(fdopendir(fd).cast::<_>());

    match set_errno(rsix::fs::Dir::from(OwnedFd::from_raw_fd(fd))) {
        Some(dir) => Box::into_raw(Box::new(dir)).cast::<_>(),
        None => null_mut(),
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
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

    let dir = dir.cast::<rsix::fs::Dir>();
    match (*dir).read() {
        None => {
            *ptr = null_mut();
            0
        }
        Some(Ok(e)) => {
            let file_type = match e.file_type() {
                rsix::fs::FileType::RegularFile => data::DT_REG,
                rsix::fs::FileType::Directory => data::DT_DIR,
                rsix::fs::FileType::Symlink => data::DT_LNK,
                rsix::fs::FileType::Fifo => data::DT_FIFO,
                rsix::fs::FileType::Socket => data::DT_SOCK,
                rsix::fs::FileType::CharacterDevice => data::DT_CHR,
                rsix::fs::FileType::BlockDevice => data::DT_BLK,
                rsix::fs::FileType::Unknown => data::DT_UNKNOWN,
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

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn closedir(dir: *mut c_void) -> c_int {
    libc!(closedir(dir.cast::<_>()));

    drop(Box::<rsix::fs::Dir>::from_raw(dir.cast()));
    0
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn dirfd(dir: *mut c_void) -> c_int {
    libc!(dirfd(dir.cast::<_>()));

    let dir = dir.cast::<rsix::fs::Dir>();
    (*dir).as_fd().as_raw_fd()
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn readdir() {
    //libc!(readdir());
    unimplemented!("readdir")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn rewinddir() {
    //libc!(rewinddir());
    unimplemented!("rewinddir")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn scandir() {
    //libc!(scandir());
    unimplemented!("scandir")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn seekdir() {
    //libc!(seekdir());
    unimplemented!("seekdir")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn telldir() {
    //libc!(telldir());
    unimplemented!("telldir")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
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
        *__errno_location() = rsix::io::Error::BADF.raw_os_error();
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
    match set_errno(rsix::fs::copy_file_range(
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

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn chmod(pathname: *const c_char, mode: c_uint) -> c_int {
    libc!(chmod(pathname, mode));

    let mode = Mode::from_bits(mode as _).unwrap();
    match set_errno(rsix::fs::chmodat(&cwd(), CStr::from_ptr(pathname), mode)) {
        Some(()) => 0,
        None => -1,
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn fchmod(fd: c_int, mode: c_uint) -> c_int {
    libc!(fchmod(fd, mode));

    let mode = Mode::from_bits(mode as _).unwrap();
    match set_errno(rsix::fs::fchmod(&BorrowedFd::borrow_raw_fd(fd), mode)) {
        Some(()) => 0,
        None => -1,
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
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
    match set_errno(rsix::fs::linkat(
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

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn symlink(target: *const c_char, linkpath: *const c_char) -> c_int {
    libc!(symlink(target, linkpath));

    match set_errno(rsix::fs::symlinkat(
        CStr::from_ptr(target),
        &cwd(),
        CStr::from_ptr(linkpath),
    )) {
        Some(()) => 0,
        None => -1,
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn chown() {
    //libc!(chown());
    unimplemented!("chown")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn lchown() {
    //libc!(lchown());
    unimplemented!("lchown")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn fchown() {
    //libc!(fchown());
    unimplemented!("fchown")
}

// net

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn accept(
    fd: c_int,
    addr: *mut SocketAddrStorage,
    len: *mut data::SockLen,
) -> c_int {
    libc!(accept(fd, same_ptr_mut(addr), len));

    match set_errno(rsix::net::acceptfrom(&BorrowedFd::borrow_raw_fd(fd))) {
        Some((accepted_fd, from)) => {
            let encoded_len = from.write(addr);
            *len = encoded_len.try_into().unwrap();
            accepted_fd.into_raw_fd()
        }
        None => -1,
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn accept4(
    fd: c_int,
    addr: *mut SocketAddrStorage,
    len: *mut data::SockLen,
    flags: c_int,
) -> c_int {
    libc!(accept4(fd, same_ptr_mut(addr), len, flags));

    let flags = AcceptFlags::from_bits(flags as _).unwrap();
    match set_errno(rsix::net::acceptfrom_with(
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

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn bind(
    sockfd: c_int,
    addr: *const SocketAddrStorage,
    len: data::SockLen,
) -> c_int {
    libc!(bind(sockfd, same_ptr(addr), len));

    let addr = match set_errno(SocketAddr::read(addr, len.try_into().unwrap())) {
        Some(addr) => addr,
        None => return -1,
    };
    match set_errno(match addr {
        SocketAddr::V4(v4) => rsix::net::bind_v4(&BorrowedFd::borrow_raw_fd(sockfd), &v4),
        SocketAddr::V6(v6) => rsix::net::bind_v6(&BorrowedFd::borrow_raw_fd(sockfd), &v6),
        SocketAddr::Unix(unix) => rsix::net::bind_unix(&BorrowedFd::borrow_raw_fd(sockfd), &unix),
        _ => panic!("unrecognized SocketAddr {:?}", addr),
    }) {
        Some(()) => 0,
        None => -1,
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn connect(
    sockfd: c_int,
    addr: *const SocketAddrStorage,
    len: data::SockLen,
) -> c_int {
    libc!(connect(sockfd, same_ptr(addr), len));

    let addr = match set_errno(SocketAddr::read(addr, len.try_into().unwrap())) {
        Some(addr) => addr,
        None => return -1,
    };
    match set_errno(match addr {
        SocketAddr::V4(v4) => rsix::net::connect_v4(&BorrowedFd::borrow_raw_fd(sockfd), &v4),
        SocketAddr::V6(v6) => rsix::net::connect_v6(&BorrowedFd::borrow_raw_fd(sockfd), &v6),
        SocketAddr::Unix(unix) => {
            rsix::net::connect_unix(&BorrowedFd::borrow_raw_fd(sockfd), &unix)
        }
        _ => panic!("unrecognized SocketAddr {:?}", addr),
    }) {
        Some(()) => 0,
        None => -1,
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn getpeername(
    fd: c_int,
    addr: *mut SocketAddrStorage,
    len: *mut data::SockLen,
) -> c_int {
    libc!(getpeername(fd, same_ptr_mut(addr), len));

    match set_errno(rsix::net::getpeername(&BorrowedFd::borrow_raw_fd(fd))) {
        Some(from) => {
            let encoded_len = from.write(addr);
            *len = encoded_len.try_into().unwrap();
            0
        }
        None => -1,
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn getsockname(
    fd: c_int,
    addr: *mut SocketAddrStorage,
    len: *mut data::SockLen,
) -> c_int {
    libc!(getsockname(fd, same_ptr_mut(addr), len));

    match set_errno(rsix::net::getsockname(&BorrowedFd::borrow_raw_fd(fd))) {
        Some(from) => {
            let encoded_len = from.write(addr);
            *len = encoded_len.try_into().unwrap();
            0
        }
        None => -1,
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn getsockopt(
    fd: c_int,
    level: c_int,
    optname: c_int,
    optval: *mut c_void,
    optlen: *mut data::SockLen,
) -> c_int {
    use rsix::net::sockopt::{self, Timeout};
    use std::time::Duration;

    unsafe fn write_bool(
        value: rsix::io::Result<bool>,
        optval: *mut c_void,
        optlen: *mut data::SockLen,
    ) -> rsix::io::Result<()> {
        Ok(write(value? as c_uint, optval.cast::<c_uint>(), optlen))
    }

    unsafe fn write_u32(
        value: rsix::io::Result<u32>,
        optval: *mut c_void,
        optlen: *mut data::SockLen,
    ) -> rsix::io::Result<()> {
        Ok(write(value?, optval.cast::<u32>(), optlen))
    }

    unsafe fn write_linger(
        linger: rsix::io::Result<Option<Duration>>,
        optval: *mut c_void,
        optlen: *mut data::SockLen,
    ) -> rsix::io::Result<()> {
        let linger = linger?;
        let linger = data::Linger {
            l_onoff: linger.is_some() as c_int,
            l_linger: linger.unwrap_or_default().as_secs() as c_int,
        };
        Ok(write(linger, optval.cast::<data::Linger>(), optlen))
    }

    unsafe fn write_timeval(
        value: rsix::io::Result<Option<Duration>>,
        optval: *mut c_void,
        optlen: *mut data::SockLen,
    ) -> rsix::io::Result<()> {
        let timeval = match value? {
            None => data::OldTimeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            Some(duration) => data::OldTimeval {
                tv_sec: duration
                    .as_secs()
                    .try_into()
                    .map_err(|_| rsix::io::Error::OVERFLOW)?,
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

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn setsockopt(
    fd: c_int,
    level: c_int,
    optname: c_int,
    optval: *const c_void,
    optlen: data::SockLen,
) -> c_int {
    use rsix::net::sockopt::{self, Timeout};
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

#[inline(never)]
#[link_section = ".text.__mustang"]
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
            *__errno_location() = rsix::io::Error::ILSEQ.raw_os_error();
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
                        let len = SocketAddr::V4(SocketAddrV4::new(v4, 0)).write(storage);
                        info.ai_addr = storage;
                        info.ai_addrlen = len.try_into().unwrap();
                    }
                    IpAddr::V6(v6) => {
                        // TODO: Create and write to `SocketAddrV6Storage`?
                        let storage = std::alloc::alloc(addr_layout).cast::<SocketAddrStorage>();
                        let len = SocketAddr::V6(SocketAddrV6::new(v6, 0, 0, 0)).write(storage);
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
            if let Some(err) = rsix::io::Error::from_io_error(&err) {
                *__errno_location() = err.raw_os_error();
                data::EAI_SYSTEM
            } else {
                // TODO: sync-resolve should return a custom error type
                if err.to_string()
                    == "failed to resolve host: server responded with error: server failure"
                {
                    data::EAI_NONAME
                } else if err.to_string()
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

#[inline(never)]
#[link_section = ".text.__mustang"]
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

#[inline(never)]
#[link_section = ".text.__mustang"]
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

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn gethostname(name: *mut c_char, len: usize) -> c_int {
    let uname = rsix::process::uname();
    let nodename = uname.nodename();
    if nodename.len() + 1 > len {
        *__errno_location() = rsix::io::Error::NAMETOOLONG.raw_os_error();
        return -1;
    }
    memcpy(
        name.cast::<_>(),
        nodename.as_bytes().as_ptr().cast::<_>(),
        nodename.len(),
    );
    *name.add(nodename.len()) = 0;
    0
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn listen(fd: c_int, backlog: c_int) -> c_int {
    libc!(listen(fd, backlog));

    match set_errno(rsix::net::listen(&BorrowedFd::borrow_raw_fd(fd), backlog)) {
        Some(()) => 0,
        None => -1,
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn recv(fd: c_int, ptr: *mut c_void, len: usize, flags: c_int) -> isize {
    libc!(recv(fd, ptr, len, flags));

    let flags = RecvFlags::from_bits(flags as _).unwrap();
    match set_errno(rsix::net::recv(
        &BorrowedFd::borrow_raw_fd(fd),
        slice::from_raw_parts_mut(ptr.cast::<u8>(), len),
        flags,
    )) {
        Some(nread) => nread as isize,
        None => -1,
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
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
    match set_errno(rsix::net::recvfrom(
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

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn send(fd: c_int, buf: *const c_void, len: usize, flags: c_int) -> isize {
    libc!(send(fd, buf, len, flags));

    let flags = SendFlags::from_bits(flags as _).unwrap();
    match set_errno(rsix::net::send(
        &BorrowedFd::borrow_raw_fd(fd),
        slice::from_raw_parts(buf.cast::<u8>(), len),
        flags,
    )) {
        Some(nwritten) => nwritten as isize,
        None => -1,
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
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
    let addr = match set_errno(SocketAddr::read(to, to_len.try_into().unwrap())) {
        Some(addr) => addr,
        None => return -1,
    };
    match set_errno(match addr {
        SocketAddr::V4(v4) => rsix::net::sendto_v4(
            &BorrowedFd::borrow_raw_fd(fd),
            slice::from_raw_parts(buf.cast::<u8>(), len),
            flags,
            &v4,
        ),
        SocketAddr::V6(v6) => rsix::net::sendto_v6(
            &BorrowedFd::borrow_raw_fd(fd),
            slice::from_raw_parts(buf.cast::<u8>(), len),
            flags,
            &v6,
        ),
        SocketAddr::Unix(unix) => rsix::net::sendto_unix(
            &BorrowedFd::borrow_raw_fd(fd),
            slice::from_raw_parts(buf.cast::<u8>(), len),
            flags,
            &unix,
        ),
        _ => panic!("unrecognized SocketAddr {:?}", addr),
    }) {
        Some(nwritten) => nwritten as isize,
        None => -1,
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn shutdown(fd: c_int, how: c_int) -> c_int {
    libc!(shutdown(fd, how));

    let how = match how {
        data::SHUT_RD => Shutdown::Read,
        data::SHUT_WR => Shutdown::Write,
        data::SHUT_RDWR => Shutdown::ReadWrite,
        _ => panic!("unrecognized shutdown kind {}", how),
    };
    match set_errno(rsix::net::shutdown(&BorrowedFd::borrow_raw_fd(fd), how)) {
        Some(()) => 0,
        None => -1,
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn socket(domain: c_int, type_: c_int, protocol: c_int) -> c_int {
    libc!(socket(domain, type_, protocol));

    let domain = AddressFamily::from_raw(domain as _);
    let flags = SocketFlags::from_bits_truncate(type_ as _);
    let type_ = SocketType::from_raw(type_ as u32 & !SocketFlags::all().bits());
    let protocol = Protocol::from_raw(protocol as _);
    match set_errno(rsix::net::socket_with(domain, type_, flags, protocol)) {
        Some(fd) => fd.into_raw_fd(),
        None => -1,
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
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
    match set_errno(rsix::net::socketpair(domain, type_, flags, protocol)) {
        Some((fd0, fd1)) => {
            (*sv) = [fd0.into_raw_fd(), fd1.into_raw_fd()];
            0
        }
        None => -1,
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn __res_init() -> c_int {
    libc!(res_init());

    unimplemented!("__res_init")
}

// io

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn write(fd: c_int, ptr: *const c_void, len: usize) -> isize {
    libc!(write(fd, ptr, len));

    match set_errno(rsix::io::write(
        &BorrowedFd::borrow_raw_fd(fd),
        slice::from_raw_parts(ptr.cast::<u8>(), len),
    )) {
        Some(nwritten) => nwritten as isize,
        None => -1,
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn writev(fd: c_int, iov: *const std::io::IoSlice, iovcnt: c_int) -> isize {
    libc!(writev(fd, same_ptr(iov), iovcnt));

    if iovcnt < 0 {
        *__errno_location() = rsix::io::Error::INVAL.raw_os_error();
        return -1;
    }

    match set_errno(rsix::io::writev(
        &BorrowedFd::borrow_raw_fd(fd),
        slice::from_raw_parts(iov, iovcnt as usize),
    )) {
        Some(nwritten) => nwritten as isize,
        None => -1,
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn read(fd: c_int, ptr: *mut c_void, len: usize) -> isize {
    libc!(read(fd, ptr, len));

    match set_errno(rsix::io::read(
        &BorrowedFd::borrow_raw_fd(fd),
        slice::from_raw_parts_mut(ptr.cast::<u8>(), len),
    )) {
        Some(nread) => nread as isize,
        None => -1,
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn readv(fd: c_int, iov: *const std::io::IoSliceMut, iovcnt: c_int) -> isize {
    libc!(readv(fd, same_ptr(iov), iovcnt));

    if iovcnt < 0 {
        *__errno_location() = rsix::io::Error::INVAL.raw_os_error();
        return -1;
    }

    match set_errno(rsix::io::readv(
        &BorrowedFd::borrow_raw_fd(fd),
        slice::from_raw_parts(iov, iovcnt as usize),
    )) {
        Some(nread) => nread as isize,
        None => -1,
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pread64(fd: c_int, ptr: *mut c_void, len: usize, offset: i64) -> isize {
    libc!(pread64(fd, ptr, len, offset));

    match set_errno(rsix::io::pread(
        &BorrowedFd::borrow_raw_fd(fd),
        slice::from_raw_parts_mut(ptr.cast::<u8>(), len),
        offset as u64,
    )) {
        Some(nread) => nread as isize,
        None => -1,
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pwrite64(fd: c_int, ptr: *const c_void, len: usize, offset: i64) -> isize {
    libc!(pwrite64(fd, ptr, len, offset));

    match set_errno(rsix::io::pwrite(
        &BorrowedFd::borrow_raw_fd(fd),
        slice::from_raw_parts(ptr.cast::<u8>(), len),
        offset as u64,
    )) {
        Some(nwritten) => nwritten as isize,
        None => -1,
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn poll(fds: *mut rsix::io::PollFd, nfds: c_ulong, timeout: c_int) -> c_int {
    libc!(poll(same_ptr_mut(fds), nfds, timeout));

    let fds = slice::from_raw_parts_mut(fds, nfds.try_into().unwrap());
    match set_errno(rsix::io::poll(fds, timeout)) {
        Some(num) => num.try_into().unwrap(),
        None => -1,
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn close(fd: c_int) -> c_int {
    libc!(close(fd));

    rsix::io::close(fd);
    0
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn dup2(fd: c_int, to: c_int) -> c_int {
    libc!(dup2(fd, to));

    let to = OwnedFd::from_raw_fd(to).into();
    match set_errno(rsix::io::dup2(&BorrowedFd::borrow_raw_fd(fd), &to)) {
        Some(()) => OwnedFd::from(to).into_raw_fd(),
        None => -1,
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn isatty(fd: c_int) -> c_int {
    libc!(isatty(fd));

    if rsix::io::isatty(&BorrowedFd::borrow_raw_fd(fd)) {
        1
    } else {
        *__errno_location() = rsix::io::Error::NOTTY.raw_os_error();
        0
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn ioctl(fd: c_int, request: c_long, mut args: ...) -> c_int {
    match request {
        data::TCGETS => {
            libc!(ioctl(fd, TCGETS));
            let fd = BorrowedFd::borrow_raw_fd(fd);
            match set_errno(rsix::io::ioctl_tcgets(&fd)) {
                Some(x) => {
                    *args.arg::<*mut rsix::io::Termios>() = x;
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
            match set_errno(rsix::io::ioctl_fionbio(&fd, value)) {
                Some(()) => 0,
                None => -1,
            }
        }
        _ => panic!("unrecognized ioctl({})", request),
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pipe2(pipefd: *mut c_int, flags: c_int) -> c_int {
    libc!(pipe2(pipefd, flags));

    let flags = PipeFlags::from_bits(flags as _).unwrap();
    match set_errno(rsix::io::pipe_with(flags)) {
        Some((a, b)) => {
            *pipefd = a.into_raw_fd();
            *pipefd.add(1) = b.into_raw_fd();
            0
        }
        None => -1,
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn sendfile() {
    //libc!(sendfile());
    unimplemented!("sendfile")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn sendmsg() {
    //libc!(sendmsg());
    unimplemented!("sendmsg")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn recvmsg() {
    //libc!(recvmsg());
    unimplemented!("recvmsg")
}

// malloc

// Keep track of every `malloc`'d pointer. This isn't amazingly efficient,
// but it works.
static MALLOC_METADATA: once_cell::sync::Lazy<
    std::sync::Mutex<std::collections::HashMap<usize, std::alloc::Layout>>,
> = once_cell::sync::Lazy::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn malloc(size: usize) -> *mut c_void {
    libc!(malloc(size));

    let layout = std::alloc::Layout::from_size_align(size, data::ALIGNOF_MAXALIGN_T).unwrap();
    let ptr = std::alloc::alloc(layout).cast::<_>();

    MALLOC_METADATA.lock().unwrap().insert(ptr as usize, layout);

    ptr
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn realloc(old: *mut c_void, size: usize) -> *mut c_void {
    libc!(realloc(old, size));

    if old.is_null() {
        malloc(size)
    } else {
        let remove = MALLOC_METADATA.lock().unwrap().remove(&(old as usize));
        let old_layout = remove.unwrap();
        if old_layout.size() >= size {
            return old;
        }

        let new = malloc(size);
        memcpy(new, old, std::cmp::min(size, old_layout.size()));
        std::alloc::dealloc(old.cast::<_>(), old_layout);
        new
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn calloc(nmemb: usize, size: usize) -> *mut c_void {
    libc!(calloc(nmemb, size));

    let product = match nmemb.checked_mul(size) {
        Some(product) => product,
        None => {
            *__errno_location() = rsix::io::Error::NOMEM.raw_os_error();
            return null_mut();
        }
    };

    let ptr = malloc(product);
    memset(ptr, 0, product)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn posix_memalign(
    memptr: *mut *mut c_void,
    alignment: usize,
    size: usize,
) -> c_int {
    libc!(posix_memalign(memptr, alignment, size));

    let layout = std::alloc::Layout::from_size_align(size, alignment).unwrap();
    let ptr = std::alloc::alloc(layout).cast::<_>();

    MALLOC_METADATA.lock().unwrap().insert(ptr as usize, layout);

    *memptr = ptr;
    0
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn free(ptr: *mut c_void) {
    libc!(free(ptr));

    if ptr.is_null() {
        return;
    }

    let remove = MALLOC_METADATA.lock().unwrap().remove(&(ptr as usize));
    let layout = remove.unwrap();
    std::alloc::dealloc(ptr.cast::<_>(), layout);
}

// mem

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
pub unsafe extern "C" fn memcmp(a: *const c_void, b: *const c_void, len: usize) -> c_int {
    libc!(memcmp(a, b, len));

    compiler_builtins::mem::memcmp(a.cast::<_>(), b.cast::<_>(), len)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
pub unsafe extern "C" fn bcmp(a: *const c_void, b: *const c_void, len: usize) -> c_int {
    // `bcmp` is just an alias for `memcmp`.
    libc!(memcmp(a, b, len));

    compiler_builtins::mem::bcmp(a.cast::<_>(), b.cast::<_>(), len)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
pub unsafe extern "C" fn memcpy(dst: *mut c_void, src: *const c_void, len: usize) -> *mut c_void {
    libc!(memcpy(dst, src, len));

    compiler_builtins::mem::memcpy(dst.cast::<_>(), src.cast::<_>(), len).cast::<_>()
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
pub unsafe extern "C" fn memmove(dst: *mut c_void, src: *const c_void, len: usize) -> *mut c_void {
    libc!(memmove(dst, src, len));

    compiler_builtins::mem::memmove(dst.cast::<_>(), src.cast::<_>(), len).cast::<_>()
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
pub unsafe extern "C" fn memset(dst: *mut c_void, fill: c_int, len: usize) -> *mut c_void {
    libc!(memset(dst, fill, len));

    compiler_builtins::mem::memset(dst.cast::<_>(), fill, len).cast::<_>()
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn memchr(s: *const c_void, c: c_int, len: usize) -> *mut c_void {
    libc!(memchr(s, c, len));

    let slice: &[u8] = slice::from_raw_parts(s.cast(), len);
    match memchr::memchr(c as u8, slice) {
        None => null_mut(),
        Some(i) => s.cast::<u8>().add(i) as *mut c_void,
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn memrchr(s: *const c_void, c: c_int, len: usize) -> *mut c_void {
    libc!(memrchr(s, c, len));

    let slice: &[u8] = slice::from_raw_parts(s.cast(), len);
    match memchr::memrchr(c as u8, slice) {
        None => null_mut(),
        Some(i) => s.cast::<u8>().add(i) as *mut c_void,
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
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

#[inline(never)]
#[link_section = ".text.__mustang"]
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
        rsix::io::mmap_anonymous(addr, length, prot, flags)
    } else {
        rsix::io::mmap(
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

#[inline(never)]
#[link_section = ".text.__mustang"]
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
        rsix::io::mmap_anonymous(addr, length, prot, flags)
    } else {
        rsix::io::mmap(
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

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn munmap(ptr: *mut c_void, len: usize) -> c_int {
    libc!(munmap(ptr, len));

    match set_errno(rsix::io::munmap(ptr, len)) {
        Some(()) => 0,
        None => -1,
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
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
        match set_errno(rsix::io::mremap_fixed(
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
        match set_errno(rsix::io::mremap(old_address, old_size, new_size, flags)) {
            Some(new_address) => new_address,
            None => data::MAP_FAILED,
        }
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn mprotect(addr: *mut c_void, length: usize, prot: c_int) -> c_int {
    libc!(mprotect(addr, length, prot));

    let prot = MprotectFlags::from_bits(prot as _).unwrap();
    match set_errno(rsix::io::mprotect(addr, length, prot)) {
        Some(()) => 0,
        None => -1,
    }
}

// rand

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn getrandom(buf: *mut c_void, buflen: usize, flags: u32) -> isize {
    libc!(getrandom(buf, buflen, flags));

    if buflen == 0 {
        return 0;
    }
    let flags = rsix::rand::GetRandomFlags::from_bits(flags).unwrap();
    match set_errno(rsix::rand::getrandom(
        slice::from_raw_parts_mut(buf.cast::<u8>(), buflen),
        flags,
    )) {
        Some(num) => num as isize,
        None => -1,
    }
}

// termios

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn tcgetattr() {
    //libc!(tcgetattr());
    unimplemented!("tcgetattr")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn tcsetattr() {
    //libc!(tcsetattr());
    unimplemented!("tcsetattr")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn cfmakeraw() {
    //libc!(cfmakeraw());
    unimplemented!("cfmakeraw")
}

// process

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn chroot(_name: *const c_char) -> c_int {
    libc!(chroot(name));
    unimplemented!("chroot")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn sysconf(name: c_int) -> c_long {
    libc!(sysconf(name));

    match name {
        data::_SC_PAGESIZE => rsix::process::page_size() as _,
        data::_SC_GETPW_R_SIZE_MAX => -1,
        // Oddly, only ever one processor seems to be online.
        data::_SC_NPROCESSORS_ONLN => 1,
        data::_SC_SYMLOOP_MAX => data::SYMLOOP_MAX,
        _ => panic!("unrecognized sysconf({})", name),
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn getcwd(buf: *mut c_char, len: usize) -> *mut c_char {
    libc!(getcwd(buf, len));

    match set_errno(rsix::process::getcwd(OsString::new())) {
        Some(path) => {
            let path = path.as_bytes();
            if path.len() + 1 <= len {
                memcpy(buf.cast(), path.as_ptr().cast(), path.len());
                *buf.add(path.len()) = 0;
                buf
            } else {
                *__errno_location() = rsix::io::Error::RANGE.raw_os_error();
                null_mut()
            }
        }
        None => null_mut(),
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn chdir(path: *const c_char) -> c_int {
    libc!(chdir(path));

    let path = CStr::from_ptr(path);
    match set_errno(rsix::process::chdir(path)) {
        Some(()) => 0,
        None => -1,
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
fn getauxval(type_: c_ulong) -> c_ulong {
    libc!(getauxval(type_));
    unimplemented!("unrecognized getauxval {}", type_)
}

#[cfg(target_arch = "aarch64")]
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
fn __getauxval(type_: c_ulong) -> c_ulong {
    libc!(getauxval(type_));
    match type_ {
        data::AT_HWCAP => rsix::process::linux_hwcap().0 as c_ulong,
        _ => unimplemented!("unrecognized __getauxval {}", type_),
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn dl_iterate_phdr(
    callback: unsafe extern "C" fn(
        info: *mut data::DlPhdrInfo,
        size: usize,
        data: *mut c_void,
    ) -> c_int,
    data: *mut c_void,
) -> c_int {
    extern "C" {
        static mut __executable_start: c_void;
    }

    libc!(dl_iterate_phdr(callback, data));

    let (phdr, phnum) = rsix::runtime::exe_phdrs();
    let mut info = data::DlPhdrInfo {
        dlpi_addr: &mut __executable_start as *mut _ as usize,
        dlpi_name: b"/proc/self/exe\0".as_ptr().cast(),
        dlpi_phdr: phdr.cast(),
        dlpi_phnum: phnum.try_into().unwrap(),
    };
    callback(&mut info, size_of::<data::DlPhdrInfo>(), data)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
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
        if CStr::from_ptr(symbol).to_bytes() == b"clone3" {
            return clone3 as *mut c_void;
        }
        if CStr::from_ptr(symbol).to_bytes() == b"copy_file_range" {
            return copy_file_range as *mut c_void;
        }
        if CStr::from_ptr(symbol).to_bytes() == b"__pthread_get_minstack" {
            // Let's just say we don't support this for now.
            return null_mut();
        }
        if CStr::from_ptr(symbol).to_bytes() == b"gnu_get_libc_version" {
            // Let's just say we don't support this for now.
            return null_mut();
        }
    }
    unimplemented!("dlsym({:?})", CStr::from_ptr(symbol))
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn dlclose() {
    //libc!(dlclose());
    unimplemented!("dlclose")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn dlerror() {
    //libc!(dlerror());
    unimplemented!("dlerror")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn dlopen() {
    //libc!(dlopen());
    unimplemented!("dlopen")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn __tls_get_addr() {
    //libc!(__tls_get_addr());
    unimplemented!("__tls_get_addr")
}

#[cfg(target_arch = "x86")]
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn ___tls_get_addr() {
    //libc!(___tls_get_addr());
    unimplemented!("___tls_get_addr")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn sched_yield() -> c_int {
    libc!(sched_yield());

    rsix::process::sched_yield();
    0
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn sched_getaffinity() {
    //libc!(sched_getaffinity());
    unimplemented!("sched_getaffinity")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
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
            match set_errno(rsix::runtime::set_thread_name(CStr::from_ptr(
                arg2 as *const c_char,
            ))) {
                Some(()) => 0,
                None => -1,
            }
        }
        _ => unimplemented!("unrecognized prctl op {}", option),
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn setgid() {
    //libc!(setgid());
    unimplemented!("setgid")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn setgroups() {
    //libc!(setgroups());
    unimplemented!("setgroups")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn setuid() {
    //libc!(setuid());
    unimplemented!("setuid")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn getpid() -> c_uint {
    libc!(getpid());
    rsix::process::getpid().as_raw()
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn getppid() -> c_uint {
    libc!(getppid());
    rsix::process::getppid().as_raw()
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn getuid() -> c_uint {
    libc!(getuid());
    rsix::process::getuid().as_raw()
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn getgid() -> c_uint {
    libc!(getgid());
    rsix::process::getgid().as_raw()
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn kill(_pid: c_uint, _sig: c_int) -> c_int {
    libc!(kill(_pid, _sig));
    unimplemented!("kill")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn raise(_sig: c_int) -> c_int {
    libc!(raise(_sig));
    unimplemented!("raise")
}

// nss

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn getpwuid_r() {
    //libc!(getpwuid_r());
    unimplemented!("getpwuid_r")
}

// signal

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn abort() {
    libc!(abort());
    unimplemented!("abort")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn signal(_num: c_int, _handler: usize) -> usize {
    libc!(signal(_num, _handler));

    // Somehow no signals for this handler were ever delivered. How odd.
    data::SIG_DFL
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn sigaction(_signum: c_int, _act: *const c_void, _oldact: *mut c_void) -> c_int {
    libc!(sigaction(_signum, _act.cast::<_>(), _oldact.cast::<_>()));

    // No signals for this handler were ever delivered either. What a coincidence.
    0
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn sigaltstack(_ss: *const c_void, _old_ss: *mut c_void) -> c_int {
    libc!(sigaltstack(_ss.cast::<_>(), _old_ss.cast::<_>()));

    // Somehow no signals were delivered and the stack wasn't ever used. What luck.
    0
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn sigaddset() {
    //libc!(sigaddset);
    unimplemented!("sigaddset")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn sigemptyset() {
    //libc!(sigemptyset);
    unimplemented!("sigemptyset")
}

// syscall

#[inline(never)]
#[link_section = ".text.__mustang"]
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
            use rsix::thread::{futex, FutexFlags, FutexOperation};

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
            let new_timespec = rsix::time::Timespec {
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
        _ => unimplemented!("syscall({:?})", number),
    }
}

// posix_spawn

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn posix_spawnp() {
    //libc!(posix_spawnp());
    unimplemented!("posix_spawnp");
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn posix_spawnattr_destroy() {
    //libc!(posix_spawnattr_destroy());
    unimplemented!("posix_spawnattr_destroy")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn posix_spawnattr_init() {
    //libc!(posix_spawnattr_init());
    unimplemented!("posix_spawnattr_init")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn posix_spawnattr_setflags() {
    //libc!(posix_spawnattr_setflags());
    unimplemented!("posix_spawnattr_setflags")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn posix_spawnattr_setsigdefault() {
    //libc!(posix_spawnattr_setsigdefault());
    unimplemented!("posix_spawnattr_setsigdefault")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn posix_spawnattr_setsigmask() {
    //libc!(posix_spawnattr_setsigmask());
    unimplemented!("posix_spawnsetsigmask")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn posix_spawn_file_actions_adddup2() {
    //libc!(posix_spawn_file_actions_adddup2());
    unimplemented!("posix_spawn_file_actions_adddup2")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn posix_spawn_file_actions_destroy() {
    //libc!(posix_spawn_file_actions_destroy());
    unimplemented!("posix_spawn_file_actions_destroy")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn posix_spawn_file_actions_init() {
    //libc!(posix_spawn_file_actions_init());
    unimplemented!("posix_spawn_file_actions_init")
}

// time

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn clock_gettime(id: c_int, tp: *mut rsix::time::Timespec) -> c_int {
    libc!(clock_gettime(id, same_ptr_mut(tp)));

    let id = match id {
        data::CLOCK_MONOTONIC => rsix::time::ClockId::Monotonic,
        data::CLOCK_REALTIME => rsix::time::ClockId::Realtime,
        _ => panic!("unimplemented clock({})", id),
    };
    *tp = rsix::time::clock_gettime(id);
    0
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn nanosleep(
    req: *const data::OldTimespec,
    rem: *mut data::OldTimespec,
) -> c_int {
    libc!(nanosleep(same_ptr(req), same_ptr_mut(rem)));

    let req = rsix::time::Timespec {
        tv_sec: (*req).tv_sec.into(),
        tv_nsec: (*req).tv_nsec as _,
    };
    match rsix::time::nanosleep(&req) {
        rsix::time::NanosleepRelativeResult::Ok => 0,
        rsix::time::NanosleepRelativeResult::Interrupted(remaining) => {
            *rem = data::OldTimespec {
                tv_sec: remaining.tv_sec.try_into().unwrap(),
                tv_nsec: remaining.tv_nsec as _,
            };
            *__errno_location() = rsix::io::Error::INTR.raw_os_error();
            -1
        }
        rsix::time::NanosleepRelativeResult::Err(err) => {
            *__errno_location() = err.raw_os_error();
            -1
        }
    }
}

// exec

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn execvp() {
    //libc!(execvp());
    unimplemented!("execvp")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn fork() {
    libc!(fork());
    unimplemented!("fork")
}

// <https://refspecs.linuxbase.org/LSB_5.0.0/LSB-Core-generic/LSB-Core-generic/baselib---register-atfork.html>
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn __register_atfork(
    _prepare: unsafe extern "C" fn(),
    _parent: unsafe extern "C" fn(),
    _child: unsafe extern "C" fn(),
    _dso_handle: *mut c_void,
) -> c_int {
    // TODO: libc!(__register_atfork(_prepare, _parent, _child, _dso_handle));

    // We don't implement `fork` yet, so there's nothing to do.

    0
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn clone3() {
    //libc!(clone3());
    unimplemented!("clone3")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn waitpid() {
    //libc!(waitpid());
    unimplemented!("waitpid")
}

// setjmp

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn siglongjmp() {
    //libc!(siglongjmp());
    unimplemented!("siglongjmp")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn __sigsetjmp() {
    //libc!(__sigsetjmp());
    unimplemented!("__sigsetjmp")
}

// math

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn acos(x: f64) -> f64 {
    libm::acos(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn acosf(x: f32) -> f32 {
    libm::acosf(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn acosh(x: f64) -> f64 {
    libm::acosh(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn acoshf(x: f32) -> f32 {
    libm::acoshf(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn asin(x: f64) -> f64 {
    libm::asin(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn asinf(x: f32) -> f32 {
    libm::asinf(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn asinh(x: f64) -> f64 {
    libm::asinh(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn asinhf(x: f32) -> f32 {
    libm::asinhf(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn atan(x: f64) -> f64 {
    libm::atan(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn atan2(x: f64, y: f64) -> f64 {
    libm::atan2(x, y)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn atan2f(x: f32, y: f32) -> f32 {
    libm::atan2f(x, y)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn atanf(x: f32) -> f32 {
    libm::atanf(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn atanh(x: f64) -> f64 {
    libm::atanh(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn atanhf(x: f32) -> f32 {
    libm::atanhf(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn cbrt(x: f64) -> f64 {
    libm::cbrt(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn cbrtf(x: f32) -> f32 {
    libm::cbrtf(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn ceil(x: f64) -> f64 {
    libm::ceil(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn ceilf(x: f32) -> f32 {
    libm::ceilf(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn copysign(x: f64, y: f64) -> f64 {
    libm::copysign(x, y)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn copysignf(x: f32, y: f32) -> f32 {
    libm::copysignf(x, y)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn cos(x: f64) -> f64 {
    libm::cos(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn cosf(x: f32) -> f32 {
    libm::cosf(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn cosh(x: f64) -> f64 {
    libm::cosh(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn coshf(x: f32) -> f32 {
    libm::coshf(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn erf(x: f64) -> f64 {
    libm::erf(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn erfc(x: f64) -> f64 {
    libm::erfc(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn erfcf(x: f32) -> f32 {
    libm::erfcf(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn erff(x: f32) -> f32 {
    libm::erff(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn exp(x: f64) -> f64 {
    libm::exp(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn exp2(x: f64) -> f64 {
    libm::exp2(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn exp2f(x: f32) -> f32 {
    libm::exp2f(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn exp10(x: f64) -> f64 {
    libm::exp10(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn exp10f(x: f32) -> f32 {
    libm::exp10f(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn expf(x: f32) -> f32 {
    libm::expf(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn expm1(x: f64) -> f64 {
    libm::expm1(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn expm1f(x: f32) -> f32 {
    libm::expm1f(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn fabs(x: f64) -> f64 {
    libm::fabs(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn fabsf(x: f32) -> f32 {
    libm::fabsf(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn fdim(x: f64, y: f64) -> f64 {
    libm::fdim(x, y)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn fdimf(x: f32, y: f32) -> f32 {
    libm::fdimf(x, y)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn floor(x: f64) -> f64 {
    libm::floor(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn floorf(x: f32) -> f32 {
    libm::floorf(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn fma(x: f64, y: f64, z: f64) -> f64 {
    libm::fma(x, y, z)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn fmaf(x: f32, y: f32, z: f32) -> f32 {
    libm::fmaf(x, y, z)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn fmax(x: f64, y: f64) -> f64 {
    libm::fmax(x, y)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn fmaxf(x: f32, y: f32) -> f32 {
    libm::fmaxf(x, y)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn fmin(x: f64, y: f64) -> f64 {
    libm::fmin(x, y)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn fminf(x: f32, y: f32) -> f32 {
    libm::fminf(x, y)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn fmod(x: f64, y: f64) -> f64 {
    libm::fmod(x, y)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn fmodf(x: f32, y: f32) -> f32 {
    libm::fmodf(x, y)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn frexp(x: f64, y: *mut i32) -> f64 {
    let (a, b) = libm::frexp(x);
    *y = b;
    a
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn frexpf(x: f32, y: *mut i32) -> f32 {
    let (a, b) = libm::frexpf(x);
    *y = b;
    a
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn hypot(x: f64, y: f64) -> f64 {
    libm::hypot(x, y)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn hypotf(x: f32, y: f32) -> f32 {
    libm::hypotf(x, y)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn ilogb(x: f64) -> i32 {
    libm::ilogb(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn ilogbf(x: f32) -> i32 {
    libm::ilogbf(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn j0(x: f64) -> f64 {
    libm::j0(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn j0f(x: f32) -> f32 {
    libm::j0f(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn j1(x: f64) -> f64 {
    libm::j1(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn j1f(x: f32) -> f32 {
    libm::j1f(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn jn(x: i32, y: f64) -> f64 {
    libm::jn(x, y)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn jnf(x: i32, y: f32) -> f32 {
    libm::jnf(x, y)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn ldexp(x: f64, y: i32) -> f64 {
    libm::ldexp(x, y)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn ldexpf(x: f32, y: i32) -> f32 {
    libm::ldexpf(x, y)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn lgamma_r(x: f64, y: *mut i32) -> f64 {
    let (a, b) = libm::lgamma_r(x);
    *y = b;
    a
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn lgammaf_r(x: f32, y: *mut i32) -> f32 {
    let (a, b) = libm::lgammaf_r(x);
    *y = b;
    a
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn log(x: f64) -> f64 {
    libm::log(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn log1p(x: f64) -> f64 {
    libm::log1p(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn log1pf(x: f32) -> f32 {
    libm::log1pf(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn log2(x: f64) -> f64 {
    libm::log2(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn log2f(x: f32) -> f32 {
    libm::log2f(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn log10(x: f64) -> f64 {
    libm::log10(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn log10f(x: f32) -> f32 {
    libm::log10f(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn logf(x: f32) -> f32 {
    libm::logf(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn modf(x: f64, y: *mut f64) -> f64 {
    let (a, b) = libm::modf(x);
    *y = b;
    a
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn modff(x: f32, y: *mut f32) -> f32 {
    let (a, b) = libm::modff(x);
    *y = b;
    a
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn nextafter(x: f64, y: f64) -> f64 {
    libm::nextafter(x, y)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn nextafterf(x: f32, y: f32) -> f32 {
    libm::nextafterf(x, y)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pow(x: f64, y: f64) -> f64 {
    libm::pow(x, y)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn powf(x: f32, y: f32) -> f32 {
    libm::powf(x, y)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn remainder(x: f64, y: f64) -> f64 {
    libm::remainder(x, y)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn remainderf(x: f32, y: f32) -> f32 {
    libm::remainderf(x, y)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn remquo(x: f64, y: f64, z: *mut i32) -> f64 {
    let (a, b) = libm::remquo(x, y);
    *z = b;
    a
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn remquof(x: f32, y: f32, z: *mut i32) -> f32 {
    let (a, b) = libm::remquof(x, y);
    *z = b;
    a
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn round(x: f64) -> f64 {
    libm::round(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn roundf(x: f32) -> f32 {
    libm::roundf(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn scalbn(x: f64, y: i32) -> f64 {
    libm::scalbn(x, y)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn scalbnf(x: f32, y: i32) -> f32 {
    libm::scalbnf(x, y)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn sin(x: f64) -> f64 {
    libm::sin(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn sincos(x: f64, y: *mut f64, z: *mut f64) {
    let (a, b) = libm::sincos(x);
    *y = a;
    *z = b;
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn sincosf(x: f32, y: *mut f32, z: *mut f32) {
    let (a, b) = libm::sincosf(x);
    *y = a;
    *z = b;
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn sinf(x: f32) -> f32 {
    libm::sinf(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn sinh(x: f64) -> f64 {
    libm::sinh(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn sinhf(x: f32) -> f32 {
    libm::sinhf(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn sqrt(x: f64) -> f64 {
    libm::sqrt(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn sqrtf(x: f32) -> f32 {
    libm::sqrtf(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn tan(x: f64) -> f64 {
    libm::tan(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn tanf(x: f32) -> f32 {
    libm::tanf(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn tanh(x: f64) -> f64 {
    libm::tanh(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn tanhf(x: f32) -> f32 {
    libm::tanhf(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn tgamma(x: f64) -> f64 {
    libm::tgamma(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn tgammaf(x: f32) -> f32 {
    libm::tgammaf(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn trunc(x: f64) -> f64 {
    libm::trunc(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn truncf(x: f32) -> f32 {
    libm::truncf(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn y0(x: f64) -> f64 {
    libm::y0(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn y0f(x: f32) -> f32 {
    libm::y0f(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn y1(x: f64) -> f64 {
    libm::y1(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn y1f(x: f32) -> f32 {
    libm::y1f(x)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn yn(x: i32, y: f64) -> f64 {
    libm::yn(x, y)
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn ynf(x: i32, y: f32) -> f32 {
    libm::ynf(x, y)
}

// Environment variables

#[inline(never)]
#[link_section = ".text.__mustang"]
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

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn setenv() {
    //libc!(setenv());
    unimplemented!("setenv")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
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

#[link_section = ".data.__mustang"]
#[no_mangle]
#[used]
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
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_self() -> PthreadT {
    libc!(pthread_self());
    origin::current_thread() as PthreadT
}

#[cfg(feature = "threads")]
#[inline(never)]
#[link_section = ".text.__mustang"]
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
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_attr_init(attr: *mut PthreadAttrT) -> c_int {
    libc!(pthread_attr_init(same_ptr_mut(attr)));
    ptr::write(attr, PthreadAttrT::default());
    0
}

#[cfg(feature = "threads")]
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_attr_destroy(attr: *mut PthreadAttrT) -> c_int {
    libc!(pthread_attr_destroy(same_ptr_mut(attr)));
    ptr::drop_in_place(attr);
    0
}

#[cfg(feature = "threads")]
#[inline(never)]
#[link_section = ".text.__mustang"]
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
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_getspecific() -> *const c_void {
    //libc!(pthread_getspecific());
    rsix::io::write(&rsix::io::stderr(), b"unimplemented: pthread_getspecific\n").ok();
    null()
}

#[cfg(feature = "threads")]
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_key_create() -> c_int {
    //libc!(pthread_key_create());
    rsix::io::write(&rsix::io::stderr(), b"unimplemented: pthread_key_create\n").ok();
    0
}

#[cfg(feature = "threads")]
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_key_delete() -> c_int {
    //libc!(pthread_key_delete());
    rsix::io::write(&rsix::io::stderr(), b"unimplemented: pthread_key_delete\n").ok();
    0
}

#[cfg(feature = "threads")]
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_mutexattr_destroy(attr: *mut PthreadMutexattrT) -> c_int {
    libc!(pthread_mutexattr_destroy(same_ptr_mut(attr)));
    ptr::drop_in_place(attr);
    0
}

#[cfg(feature = "threads")]
#[inline(never)]
#[link_section = ".text.__mustang"]
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
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_mutexattr_settype(attr: *mut PthreadMutexattrT, kind: c_int) -> c_int {
    libc!(pthread_mutexattr_settype(same_ptr_mut(attr), kind));
    (*attr).kind = AtomicU32::new(kind as u32);
    0
}

#[cfg(feature = "threads")]
#[inline(never)]
#[link_section = ".text.__mustang"]
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
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_mutex_init(
    mutex: *mut PthreadMutexT,
    mutexattr: *const PthreadMutexattrT,
) -> c_int {
    libc!(pthread_mutex_init(same_ptr_mut(mutex), same_ptr(mutexattr)));
    let kind = (*mutexattr).kind.load(SeqCst);
    match kind as i32 {
        data::PTHREAD_MUTEX_NORMAL => ptr::write(&mut (*mutex).u.normal, RawMutex::new()),
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
#[inline(never)]
#[link_section = ".text.__mustang"]
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
#[inline(never)]
#[link_section = ".text.__mustang"]
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
        rsix::io::Error::BUSY.raw_os_error()
    }
}

#[cfg(feature = "threads")]
#[inline(never)]
#[link_section = ".text.__mustang"]
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
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_rwlock_rdlock(rwlock: *mut PthreadRwlockT) -> c_int {
    libc!(pthread_rwlock_rdlock(same_ptr_mut(rwlock)));
    (*rwlock).lock.lock_shared();
    (*rwlock).exclusive.store(false, SeqCst);
    0
}

#[cfg(feature = "threads")]
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_rwlock_unlock(rwlock: *mut PthreadRwlockT) -> c_int {
    libc!(pthread_rwlock_unlock(same_ptr_mut(rwlock)));
    if (*rwlock).exclusive.load(SeqCst) {
        (*rwlock).lock.lock_exclusive();
    } else {
        (*rwlock).lock.lock_shared();
    }
    0
}

#[cfg(feature = "threads")]
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_setspecific() -> c_int {
    //libc!(pthread_setspecific());
    rsix::io::write(&rsix::io::stderr(), b"unimplemented: pthread_getspecific\n").ok();
    0
}

#[cfg(feature = "threads")]
#[inline(never)]
#[link_section = ".text.__mustang"]
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
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_attr_setguardsize(attr: *mut PthreadAttrT, guardsize: usize) -> c_int {
    // TODO: libc!(pthread_attr_setguardsize(attr, guardsize));
    (*attr).guard_size = guardsize;
    0
}

#[cfg(feature = "threads")]
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_condattr_destroy() -> c_int {
    //libc!(pthread_condattr_destroy());
    rsix::io::write(
        &rsix::io::stderr(),
        b"unimplemented: pthread_condattr_destroy\n",
    )
    .ok();
    0
}

#[cfg(feature = "threads")]
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_condattr_init() -> c_int {
    //libc!(pthread_condattr_init());
    rsix::io::write(
        &rsix::io::stderr(),
        b"unimplemented: pthread_condattr_init\n",
    )
    .ok();
    0
}

#[cfg(feature = "threads")]
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_condattr_setclock() -> c_int {
    //libc!(pthread_condattr_setclock());
    rsix::io::write(
        &rsix::io::stderr(),
        b"unimplemented: pthread_condattr_setclock\n",
    )
    .ok();
    0
}

#[cfg(feature = "threads")]
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_cond_broadcast() -> c_int {
    //libc!(pthread_cond_broadcast());
    rsix::io::write(
        &rsix::io::stderr(),
        b"unimplemented: pthread_cond_broadcast\n",
    )
    .ok();
    0
}

#[cfg(feature = "threads")]
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_cond_destroy() -> c_int {
    //libc!(pthread_cond_destroy());
    rsix::io::write(
        &rsix::io::stderr(),
        b"unimplemented: pthread_cond_destroy\n",
    )
    .ok();
    0
}

#[cfg(feature = "threads")]
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_cond_init() -> c_int {
    //libc!(pthread_cond_init());
    rsix::io::write(&rsix::io::stderr(), b"unimplemented: pthread_cond_init\n").ok();
    0
}

#[cfg(feature = "threads")]
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_cond_signal() -> c_int {
    //libc!(pthread_cond_signal());
    rsix::io::write(&rsix::io::stderr(), b"unimplemented: pthread_cond_signal\n").ok();
    0
}

#[cfg(feature = "threads")]
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_cond_wait() -> c_int {
    //libc!(pthread_cond_wait());
    rsix::io::write(&rsix::io::stderr(), b"unimplemented: pthread_cond_wait\n").ok();
    0
}

#[cfg(feature = "threads")]
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_cond_timedwait() {
    //libc!(pthread_cond_timedwait());
    rsix::io::write(
        &rsix::io::stderr(),
        b"unimplemented: pthread_cond_timedwait\n",
    )
    .ok();
}

#[cfg(feature = "threads")]
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_rwlock_destroy(rwlock: *mut PthreadRwlockT) -> c_int {
    libc!(pthread_rwlock_destroy(same_ptr_mut(rwlock)));
    ptr::drop_in_place(rwlock);
    0
}

#[cfg(feature = "threads")]
#[inline(never)]
#[link_section = ".text.__mustang"]
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
#[inline(never)]
#[link_section = ".text.__mustang"]
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

    let thread = match origin::create_thread(fn_, arg, stack_size, guard_size) {
        Ok(thread) => thread,
        Err(e) => return e.raw_os_error(),
    };

    pthread.write(thread as PthreadT);
    0
}

#[cfg(feature = "threads")]
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_detach(pthread: PthreadT) -> c_int {
    libc!(pthread_detach(pthread));
    origin::detach_thread(pthread as *mut Thread);
    0
}

#[cfg(feature = "threads")]
#[inline(never)]
#[link_section = ".text.__mustang"]
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
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_sigmask() -> c_int {
    //libc!(pthread_sigmask());
    rsix::io::write(&rsix::io::stderr(), b"unimplemented: pthread_sigmask\n").ok();
    0
}

#[cfg(feature = "threads")]
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_attr_setstacksize(attr: *mut PthreadAttrT, stacksize: usize) -> c_int {
    libc!(pthread_attr_setstacksize(same_ptr_mut(attr), stacksize));
    (*attr).stack_size = stacksize;
    0
}

#[cfg(feature = "threads")]
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_cancel() -> c_int {
    //libc!(pthread_cancel());
    unimplemented!("pthread_cancel")
}

#[cfg(feature = "threads")]
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_exit() -> c_int {
    //libc!(pthread_exit());
    // As with `pthread_cancel`, `pthread_exit` may be tricky to implement.
    unimplemented!("pthread_exit")
}

#[cfg(feature = "threads")]
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_cleanup_push() -> c_int {
    //libc!(pthread_cleanup_push());
    unimplemented!("pthread_cleanup_push")
}

#[cfg(feature = "threads")]
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_cleanup_pop() -> c_int {
    //libc!(pthread_cleanup_pop());
    unimplemented!("pthread_cleanup_pop")
}

#[cfg(feature = "threads")]
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_setcancelstate() -> c_int {
    //libc!(pthread_setcancelstate());
    unimplemented!("pthread_setcancelstate")
}

#[cfg(feature = "threads")]
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_setcanceltype() -> c_int {
    //libc!(pthread_setcanceltype());
    unimplemented!("pthread_setcanceltype")
}

#[cfg(feature = "threads")]
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_testcancel() -> c_int {
    //libc!(pthread_testcancel());
    unimplemented!("pthread_testcancel")
}

#[cfg(feature = "threads")]
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_atfork(
    _prepare: unsafe extern "C" fn(),
    _parent: unsafe extern "C" fn(),
    _child: unsafe extern "C" fn(),
) -> c_int {
    libc!(pthread_atfork(_prepare, _parent, _child));

    // We don't implement `fork` yet, so there's nothing to do.

    0
}

#[cfg(feature = "threads")]
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn __cxa_thread_atexit_impl(
    func: unsafe extern "C" fn(*mut c_void),
    obj: *mut c_void,
    _dso_symbol: *mut c_void,
) -> c_int {
    // TODO: libc!(__cxa_thread_atexit_impl(func, obj, _dso_symbol));
    origin::at_thread_exit(func, obj);
    0
}

// exit

/// Register a function to be called when `exit` is called.
///
/// This function conforms to the [LSB `__cxa_atexit`] ABI.
///
/// [LSB `__cxa_atexit`]: <https://refspecs.linuxbase.org/LSB_5.0.0/LSB-Core-generic/LSB-Core-generic/baselib---cxa-atexit.html>
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn __cxa_atexit(
    func: unsafe extern "C" fn(*mut c_void),
    arg: *mut c_void,
    _dso: *mut c_void,
) -> c_int {
    origin::at_exit(func, arg);
    0
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn __cxa_finalize(_d: *mut c_void) {}

/// C-compatible `atexit`. Consider using `__cxa_atexit` instead.
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn atexit(func: unsafe extern "C" fn()) -> c_int {
    libc!(atexit(status));

    /// Adapter to let `atexit`-style functions be called in the same manner as
    /// `__cxa_atexit` functions.
    unsafe extern "C" fn adapter(func: *mut c_void) {
        transmute::<_, unsafe extern "C" fn()>(func)();
    }

    origin::at_exit(adapter, func as *mut c_void);
    0
}

/// C-compatible `exit`.
///
/// Call all the registered at-exit functions, and exit the program.
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
extern "C" fn exit(status: c_int) -> ! {
    libc!(exit(status));
    origin::exit(status)
}

/// POSIX-compatible `_exit`.
///
/// Just exit the process.
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
extern "C" fn _exit(status: c_int) -> ! {
    libc!(_exit(status));
    origin::exit_immediately(status)
}

// utilities

fn set_errno<T>(result: Result<T, rsix::io::Error>) -> Option<T> {
    result
        .map_err(|err| unsafe {
            *__errno_location() = err.raw_os_error();
        })
        .ok()
}
