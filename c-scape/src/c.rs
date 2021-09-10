//! Selected libc-compatible interfaces.
//!
//! The goal here isn't necessarily to build a complete libc; it's primarily
//! to provide things that `std` and possibly popular crates are currently
//! using.
//!
//! This effectively undoes the work that `rsix` does: it calls `rsix` and
//! translates it back into a C-like ABI. Ideally, Rust code should just call
//! the `rsix` APIs directly, which are safer, more ergonomic, and skip this
//! whole layer.

use crate::data;
use memoffset::offset_of;
use rsix::fs::{cwd, openat, AtFlags, FdFlags, Mode, OFlags};
#[cfg(debug_assertions)]
use rsix::io::stderr;
use rsix::io::{MapFlags, MprotectFlags, PipeFlags, ProtFlags};
use rsix::io_lifetimes::{AsFd, BorrowedFd, OwnedFd};
use std::cmp::Ordering;
use std::convert::TryInto;
use std::ffi::{c_void, CStr, OsStr};
use std::os::raw::{c_char, c_int, c_long, c_uint, c_ulong};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd};
use std::ptr::{null, null_mut};
use std::slice;

// errno

#[no_mangle]
pub unsafe extern "C" fn __errno_location() -> *mut c_int {
    // When threads are supported, this will need to return a per-thread
    // pointer, but for now, we can just return a single static address.
    static mut ERRNO: i32 = 0;
    &mut ERRNO
}

#[no_mangle]
pub unsafe extern "C" fn __xpg_strerror_r(errnum: c_int, buf: *mut c_char, buflen: usize) -> c_int {
    let message = match crate::error_str::error_str(rsix::io::Error::from_raw_os_error(errnum)) {
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
pub unsafe extern "C" fn open64(pathname: *const c_char, flags: c_int, mode: c_int) -> c_int {
    let flags = OFlags::from_bits(flags as _).unwrap();
    let mode = Mode::from_bits(mode as _).unwrap();
    match set_errno(openat(&cwd(), CStr::from_ptr(pathname), flags, mode)) {
        Some(fd) => fd.into_raw_fd(),
        None => return -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn open() {
    unimplemented!("open")
}

#[no_mangle]
pub unsafe extern "C" fn readlink(
    pathname: *const c_char,
    buf: *mut c_char,
    bufsiz: usize,
) -> isize {
    let path = match set_errno(rsix::fs::readlinkat(
        &cwd(),
        CStr::from_ptr(pathname),
        std::ffi::OsString::new(),
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
pub unsafe extern "C" fn stat() -> c_int {
    unimplemented!("stat")
}

#[no_mangle]
pub unsafe extern "C" fn stat64(pathname: *const c_char, stat: *mut rsix::fs::Stat) -> c_int {
    match set_errno(rsix::fs::statat(
        &cwd(),
        CStr::from_ptr(pathname),
        AtFlags::empty(),
    )) {
        Some(r) => {
            *stat = r;
            0
        }
        None => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn fstat(_fd: c_int, _stat: *mut rsix::fs::Stat) -> c_int {
    unimplemented!("fstat")
}

#[no_mangle]
pub unsafe extern "C" fn fstat64(fd: c_int, stat: *mut rsix::fs::Stat) -> c_int {
    match set_errno(rsix::fs::fstat(&BorrowedFd::borrow_raw_fd(fd))) {
        Some(r) => {
            *stat = r;
            0
        }
        None => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn statx(
    dirfd: c_int,
    path: *const c_char,
    flags: c_int,
    mask: c_uint,
    stat: *mut rsix::fs::Statx,
) -> c_int {
    if path.is_null() || stat.is_null() {
        *__errno_location() = rsix::io::Error::FAULT.raw_os_error();
        return -1;
    }

    let flags = AtFlags::from_bits(flags as _).unwrap();
    let mask = rsix::fs::StatxFlags::from_bits(mask).unwrap();
    match set_errno(rsix::fs::statx(
        &BorrowedFd::borrow_raw_fd(dirfd),
        CStr::from_ptr(path),
        flags,
        mask,
    )) {
        Some(r) => {
            *stat = r;
            0
        }
        None => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn realpath(path: *const c_char, resolved_path: *mut c_char) -> *mut c_char {
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

#[no_mangle]
pub unsafe extern "C" fn fcntl(fd: c_int, cmd: c_int, mut args: ...) -> c_int {
    let fd = BorrowedFd::borrow_raw_fd(fd);
    match cmd {
        data::F_GETFL => match set_errno(rsix::fs::fcntl_getfl(&fd)) {
            Some(flags) => flags.bits() as _,
            None => -1,
        }
        data::F_SETFD => match set_errno(rsix::fs::fcntl_setfd(
            &fd,
            FdFlags::from_bits(args.arg::<c_int>() as _).unwrap(),
        )) {
            Some(()) => 0,
            None => -1,
        },
        data::F_DUPFD_CLOEXEC => match set_errno(rsix::fs::fcntl_dupfd_cloexec(&fd, args.arg::<c_int>()))
        {
            Some(fd) => fd.into_raw_fd(),
            None => -1,
        },
        _ => panic!("unrecognized fnctl({})", cmd),
    }
}

#[no_mangle]
pub unsafe extern "C" fn mkdir(pathname: *const c_char, mode: c_int) -> c_int {
    let mode = Mode::from_bits(mode as _).unwrap();
    match set_errno(rsix::fs::mkdirat(&cwd(), CStr::from_ptr(pathname), mode)) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn fdatasync(fd: c_int) -> c_int {
    match set_errno(rsix::fs::fdatasync(&BorrowedFd::borrow_raw_fd(fd))) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn fstatat64(
    fd: c_int,
    pathname: *const c_char,
    stat: *mut rsix::fs::Stat,
) -> c_int {
    match set_errno(rsix::fs::statat(
        &BorrowedFd::borrow_raw_fd(fd),
        CStr::from_ptr(pathname),
        AtFlags::empty(),
    )) {
        Some(r) => {
            *stat = r;
            0
        }
        None => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn fsync(fd: c_int) -> c_int {
    match set_errno(rsix::fs::fdatasync(&BorrowedFd::borrow_raw_fd(fd))) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn ftruncate64(fd: c_int, length: i64) -> c_int {
    match set_errno(rsix::fs::ftruncate(
        &BorrowedFd::borrow_raw_fd(fd),
        length as u64,
    )) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn rename(old: *const c_char, new: *const c_char) -> c_int {
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

#[no_mangle]
pub unsafe extern "C" fn rmdir(pathname: *const c_char) -> c_int {
    match set_errno(rsix::fs::unlinkat(
        &cwd(),
        CStr::from_ptr(pathname),
        AtFlags::REMOVEDIR,
    )) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn unlink(pathname: *const c_char) -> c_int {
    match set_errno(rsix::fs::unlinkat(
        &cwd(),
        CStr::from_ptr(pathname),
        AtFlags::empty(),
    )) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn lseek64(fd: c_int, offset: i64, whence: c_int) -> i64 {
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

#[no_mangle]
pub unsafe extern "C" fn lstat64(pathname: *const c_char, stat: *mut rsix::fs::Stat) -> c_int {
    match set_errno(rsix::fs::statat(
        &cwd(),
        CStr::from_ptr(pathname),
        AtFlags::SYMLINK_NOFOLLOW,
    )) {
        Some(r) => {
            *stat = r;
            0
        }
        None => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn opendir(pathname: *const c_char) -> *mut rsix::fs::Dir {
    match set_errno(rsix::fs::openat(
        &cwd(),
        CStr::from_ptr(pathname),
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
        Mode::empty(),
    )) {
        Some(fd) => fdopendir(fd.into_raw_fd()),
        None => return null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn fdopendir(fd: c_int) -> *mut rsix::fs::Dir {
    let dir = match set_errno(rsix::fs::Dir::from(OwnedFd::from_raw_fd(fd))) {
        Some(dir) => dir,
        None => return null_mut(),
    };
    Box::into_raw(Box::new(dir))
}

#[cfg(target_env = "gnu")]
#[repr(C)]
pub struct Dirent64 {
    d_ino: u64,
    d_off: u64,
    d_reclen: u16,
    d_type: u8,
    d_name: [u8; 256],
}

#[no_mangle]
pub unsafe extern "C" fn readdir64_r(
    dir: *mut rsix::fs::Dir,
    entry: *mut Dirent64,
    ptr: *mut *mut Dirent64,
) -> c_int {
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
            *entry = Dirent64 {
                d_ino: e.ino(),
                d_off: 0, // We don't implement `seekdir` yet anyway.
                d_reclen: (offset_of!(Dirent64, d_name) + e.file_name().to_bytes().len() + 1)
                    .try_into()
                    .unwrap(),
                d_type: file_type,
                d_name: [0u8; 256],
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
pub unsafe extern "C" fn closedir(dir: *mut rsix::fs::Dir) -> c_int {
    drop(Box::from_raw(dir));
    0
}

#[no_mangle]
pub unsafe extern "C" fn dirfd(dir: *mut rsix::fs::Dir) -> c_int {
    (*dir).as_fd().as_raw_fd()
}

#[no_mangle]
pub unsafe extern "C" fn readdir() {
    unimplemented!("readdir")
}

#[no_mangle]
pub unsafe extern "C" fn rewinddir() {
    unimplemented!("rewinddir")
}

#[no_mangle]
pub unsafe extern "C" fn scandir() {
    unimplemented!("scandir")
}

#[no_mangle]
pub unsafe extern "C" fn seekdir() {
    unimplemented!("seekdir")
}

#[no_mangle]
pub unsafe extern "C" fn telldir() {
    unimplemented!("telldir")
}

#[no_mangle]
pub unsafe extern "C" fn copy_file_range(
    fd_in: c_int,
    off_in: *mut i64,
    fd_out: c_int,
    off_out: *mut i64,
    len: usize,
    flags: c_uint,
) -> isize {
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

#[no_mangle]
unsafe extern "C" fn chmod(pathname: *const c_char, mode: c_int) -> c_int {
    let mode = Mode::from_bits(mode as _).unwrap();
    match set_errno(rsix::fs::chmodat(&cwd(), CStr::from_ptr(pathname), mode)) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn fchmod(fd: c_int, mode: c_int) -> c_int {
    let mode = Mode::from_bits(mode as _).unwrap();
    match set_errno(rsix::fs::fchmod(&BorrowedFd::borrow_raw_fd(fd), mode)) {
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

#[no_mangle]
unsafe extern "C" fn symlink(target: *const c_char, linkpath: *const c_char) -> c_int {
    match set_errno(rsix::fs::symlinkat(
        CStr::from_ptr(target),
        &cwd(),
        CStr::from_ptr(linkpath),
    )) {
        Some(()) => 0,
        None => -1,
    }
}

// io

#[no_mangle]
pub unsafe extern "C" fn write(fd: c_int, ptr: *const c_void, len: usize) -> isize {
    // For now, print a message, so that we know we're doing something. We'll
    // probably remove this at some point, but for now, things are fragile
    // enough that it's nice to have this confirmation.
    #[cfg(debug_assertions)]
    if fd != stderr().as_raw_fd() {
        eprintln!(".｡oO(I/O performed by c-scape using rsix! 🌊)");
    }

    match set_errno(rsix::io::write(
        &BorrowedFd::borrow_raw_fd(fd),
        slice::from_raw_parts(ptr.cast::<u8>(), len),
    )) {
        Some(nwritten) => nwritten as isize,
        None => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn writev(fd: c_int, iov: *const std::io::IoSlice, iovcnt: c_int) -> isize {
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

#[no_mangle]
pub unsafe extern "C" fn read(fd: c_int, ptr: *mut c_void, len: usize) -> isize {
    // For now, print a message, so that we know we're doing something. We'll
    // probably remove this at some point, but for now, things are fragile
    // enough that it's nice to have this confirmation.
    #[cfg(debug_assertions)]
    if fd != stderr().as_raw_fd() {
        eprintln!(".｡oO(I/O performed by c-scape using rsix! 🌊)");
    }

    match set_errno(rsix::io::read(
        &BorrowedFd::borrow_raw_fd(fd),
        slice::from_raw_parts_mut(ptr.cast::<u8>(), len),
    )) {
        Some(nread) => nread as isize,
        None => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn readv(fd: c_int, iov: *const std::io::IoSliceMut, iovcnt: c_int) -> isize {
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

#[no_mangle]
pub unsafe extern "C" fn pread64(fd: c_int, ptr: *mut c_void, len: usize, offset: i64) -> isize {
    match set_errno(rsix::io::pread(
        &BorrowedFd::borrow_raw_fd(fd),
        slice::from_raw_parts_mut(ptr.cast::<u8>(), len),
        offset as u64,
    )) {
        Some(nread) => nread as isize,
        None => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn pwrite64(fd: c_int, ptr: *const c_void, len: usize, offset: i64) -> isize {
    match set_errno(rsix::io::pwrite(
        &BorrowedFd::borrow_raw_fd(fd),
        slice::from_raw_parts(ptr.cast::<u8>(), len),
        offset as u64,
    )) {
        Some(nwritten) => nwritten as isize,
        None => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn poll(fds: *mut rsix::io::PollFd, nfds: c_ulong, timeout: c_int) -> c_int {
    let fds = slice::from_raw_parts_mut(fds, nfds.try_into().unwrap());
    match set_errno(rsix::io::poll(fds, timeout)) {
        Some(num) => num.try_into().unwrap(),
        None => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn close(fd: c_int) -> c_int {
    rsix::io::close(fd);
    0
}

#[no_mangle]
pub unsafe extern "C" fn dup2(fd: c_int, to: c_int) -> c_int {
    let to = OwnedFd::from_raw_fd(to).into();
    match set_errno(rsix::io::dup2(&BorrowedFd::borrow_raw_fd(fd), &to)) {
        Some(()) => OwnedFd::from(to).into_raw_fd(),
        None => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn isatty(fd: c_int) -> c_int {
    if rsix::io::isatty(&BorrowedFd::borrow_raw_fd(fd)) {
        1
    } else {
        *__errno_location() = rsix::io::Error::NOTTY.raw_os_error();
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn ioctl(fd: c_int, request: c_long, mut args: ...) -> c_int {
    let fd = BorrowedFd::borrow_raw_fd(fd);
    match request {
        data::TCGETS => match set_errno(rsix::io::ioctl_tcgets(&fd)) {
            Some(x) => {
                *args.arg::<*mut rsix::io::Termios>() = x;
                0
            }
            None => -1,
        },
        _ => panic!("unrecognized ioctl({})", request),
    }
}

#[no_mangle]
pub unsafe extern "C" fn pipe2(pipefd: *mut c_int, flags: c_int) -> c_int {
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

// malloc

#[no_mangle]
pub unsafe extern "C" fn malloc(size: usize) -> *mut c_void {
    let layout = std::alloc::Layout::from_size_align(size, data::SIZEOF_MAXALIGN_T).unwrap();
    std::alloc::alloc(layout).cast::<_>()
}

#[no_mangle]
pub unsafe extern "C" fn realloc(ptr: *mut c_void, size: usize) -> *mut c_void {
    // Somehow we never end up reusing `ptr`.
    let new = malloc(size);
    memcpy(new, ptr, size);
    free(ptr);
    new
}

#[no_mangle]
pub unsafe extern "C" fn calloc(nmemb: usize, size: usize) -> *mut c_void {
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

#[no_mangle]
pub unsafe extern "C" fn posix_memalign(
    memptr: *mut *mut c_void,
    alignment: usize,
    size: usize,
) -> c_int {
    // Note that we don't currently record `alignment` anywhere. This is only
    // safe because our `free` doesn't actually call `dealloc`.
    let layout = std::alloc::Layout::from_size_align(size, alignment).unwrap();
    *memptr = std::alloc::alloc(layout).cast::<_>();
    0
}

#[no_mangle]
pub unsafe extern "C" fn free(_ptr: *mut c_void) {
    // Somehow we don't actually release any resources.
}

// mem

#[no_mangle]
pub unsafe extern "C" fn memchr(s: *mut c_void, c: c_int, len: usize) -> *mut c_void {
    for i in 0..len {
        if *s.cast::<u8>().add(i) == c as u8 {
            return s.cast::<u8>().add(i).cast::<c_void>();
        }
    }
    null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn memrchr(s: *mut c_void, c: c_int, len: usize) -> *mut c_void {
    for i in 0..len {
        if *s.cast::<u8>().add(len - i - 1) == c as u8 {
            return s.cast::<u8>().add(len - i - 1).cast::<c_void>();
        }
    }
    null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn memcmp(a: *const c_void, b: *const c_void, len: usize) -> c_int {
    for i in 0..len {
        match (*a.cast::<u8>().add(i)).cmp(&*b.cast::<u8>().add(i)) {
            Ordering::Less => return -1,
            Ordering::Greater => return 1,
            Ordering::Equal => (),
        }
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn bcmp(a: *const c_void, b: *const c_void, len: usize) -> c_int {
    memcmp(a, b, len)
}

#[no_mangle]
pub unsafe extern "C" fn memcpy(dst: *mut c_void, src: *const c_void, len: usize) -> *mut c_void {
    memcpy_forward(dst, src, len)
}

#[no_mangle]
pub unsafe extern "C" fn memmove(dst: *mut c_void, src: *const c_void, len: usize) -> *mut c_void {
    if (dst as usize) < (src as usize) {
        memcpy_forward(dst, src, len)
    } else {
        memcpy_backward(dst, src, len)
    }
}

unsafe fn memcpy_forward(dst: *mut c_void, src: *const c_void, len: usize) -> *mut c_void {
    // Avoid using `0..len` because that could generate a `memcpy`.
    let mut i = 0;
    while i != len {
        *dst.cast::<u8>().add(i) = *src.cast::<u8>().add(i);
        i += 1;
    }
    dst
}

unsafe fn memcpy_backward(dst: *mut c_void, src: *const c_void, len: usize) -> *mut c_void {
    // Avoid using `0..len` because that could generate a `memcpy`.
    let mut i = 0;
    while i != len {
        *dst.cast::<u8>().add(len - i - 1) = *src.cast::<u8>().add(len - i - 1);
        i += 1;
    }
    dst
}

#[no_mangle]
pub unsafe extern "C" fn memset(dst: *mut c_void, fill: c_int, len: usize) -> *mut c_void {
    // Avoid using `0..len` because that could generate a `memset`.
    let mut i = 0;
    while i != len {
        *dst.cast::<u8>().add(i) = fill as u8;
        i += 1;
    }
    dst
}

#[no_mangle]
pub unsafe extern "C" fn strlen(mut s: *const c_char) -> usize {
    let mut len = 0;
    while *s != b'\0' as _ {
        len += 1;
        s = s.add(1);
    }
    len
}

// mmap

#[no_mangle]
pub unsafe extern "C" fn mmap(
    addr: *mut c_void,
    length: usize,
    prot: c_int,
    flags: c_int,
    fd: c_int,
    offset: i64,
) -> *mut c_void {
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

#[no_mangle]
pub unsafe extern "C" fn munmap(ptr: *mut c_void, len: usize) -> c_int {
    match set_errno(rsix::io::munmap(ptr, len)) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn mprotect(addr: *mut c_void, length: usize, prot: c_int) -> c_int {
    let prot = MprotectFlags::from_bits(prot as _).unwrap();
    match set_errno(rsix::io::mprotect(addr, length, prot)) {
        Some(()) => 0,
        None => -1,
    }
}

// rand

#[no_mangle]
pub unsafe extern "C" fn getrandom(buf: *mut c_void, buflen: usize, flags: u32) -> isize {
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

// process

#[no_mangle]
pub unsafe extern "C" fn sysconf(name: c_int) -> c_long {
    match name {
        data::_SC_PAGESIZE => rsix::process::page_size() as _,
        data::_SC_GETPW_R_SIZE_MAX => -1,
        // Oddly, only ever one processor seems to be online.
        data::_SC_NPROCESSORS_ONLN => 1 as _,
        data::_SC_SYMLOOP_MAX => data::SYMLOOP_MAX,
        _ => panic!("unrecognized sysconf({})", name),
    }
}

#[no_mangle]
pub unsafe extern "C" fn getcwd(buf: *mut c_char, len: usize) -> *mut c_char {
    // For some reason, we always seem to be in the root directory.
    if len < 2 {
        return null_mut();
    }
    memcpy(buf.cast::<_>(), b"/\0".as_ptr().cast::<_>(), 2).cast::<_>()
}

#[no_mangle]
pub unsafe extern "C" fn chdir(_path: *const c_char) -> c_int {
    unimplemented!("chdir")
}

#[no_mangle]
pub unsafe extern "C" fn dl_iterate_phdr() {
    unimplemented!("dl_iterate_phdr")
}

#[no_mangle]
pub unsafe extern "C" fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void {
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

#[no_mangle]
pub unsafe extern "C" fn dlclose() {
    unimplemented!("dlclose")
}

#[no_mangle]
pub unsafe extern "C" fn dlerror() {
    unimplemented!("dlerror")
}

#[no_mangle]
pub unsafe extern "C" fn dlopen() {
    unimplemented!("dlopen")
}

#[no_mangle]
pub unsafe extern "C" fn __tls_get_addr() {
    unimplemented!("__tls_get_addr")
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "C" fn ___tls_get_addr() {
    unimplemented!("___tls_get_addr")
}

#[no_mangle]
pub unsafe extern "C" fn __cxa_thread_atexit_impl(
    _func: unsafe extern "C" fn(*mut c_void),
    _obj: *mut c_void,
    _dso_symbol: *mut c_void,
) -> c_int {
    // Somehow we never got around to calling the destructor. Gosh.
    0
}

#[no_mangle]
pub unsafe extern "C" fn sched_yield() -> c_int {
    rsix::process::sched_yield();
    0
}

#[no_mangle]
pub unsafe extern "C" fn sched_getaffinity() {
    unimplemented!("sched_getaffinity")
}

#[no_mangle]
pub unsafe extern "C" fn prctl() {
    unimplemented!("prctl")
}

#[no_mangle]
pub unsafe extern "C" fn setgid() {
    unimplemented!("setgid")
}

#[no_mangle]
pub unsafe extern "C" fn setgroups() {
    unimplemented!("setgroups")
}

#[no_mangle]
pub unsafe extern "C" fn setuid() {
    unimplemented!("setuid")
}

#[no_mangle]
pub unsafe extern "C" fn getpid() -> c_uint {
    rsix::process::getpid().as_raw()
}

#[no_mangle]
pub unsafe extern "C" fn getuid() -> c_uint {
    rsix::process::getuid().as_raw()
}

#[no_mangle]
pub unsafe extern "C" fn getgid() -> c_uint {
    rsix::process::getgid().as_raw()
}

// nss

#[no_mangle]
pub unsafe extern "C" fn getpwuid_r() {
    unimplemented!("getpwuid_r")
}

// signal

#[no_mangle]
pub unsafe extern "C" fn abort() {
    unimplemented!("abort")
}

#[no_mangle]
pub unsafe extern "C" fn signal(
    _num: c_int,
    _handler: unsafe extern "C" fn(c_int),
) -> *const c_void {
    // Somehow no signals for this handler were ever delivered. How odd.
    // Return `SIG_DFL`.
    null()
}

#[no_mangle]
pub unsafe extern "C" fn sigaction(
    _signum: c_int,
    _act: *const c_void,
    _oldact: *mut c_void,
) -> c_int {
    // No signals for this handler were ever delivered either. What a coincidence.
    0
}

#[no_mangle]
pub unsafe extern "C" fn sigaltstack(_ss: *const c_void, _old_ss: *mut c_void) -> c_int {
    // Somehow no signals were delivered and the stack wasn't ever used. What luck.
    0
}

#[no_mangle]
pub unsafe extern "C" fn sigaddset() {
    unimplemented!("sigaddset")
}

#[no_mangle]
pub unsafe extern "C" fn sigemptyset() {
    unimplemented!("sigemptyset")
}

// syscall

#[no_mangle]
pub unsafe extern "C" fn syscall(number: c_long, mut args: ...) -> c_long {
    match number {
        data::SYS_getrandom => getrandom(
            args.arg::<*mut c_void>(),
            args.arg::<usize>(),
            args.arg::<u32>(),
        ) as _,
        _ => unimplemented!("syscall({:?})", number),
    }
}

// posix_spawn

#[no_mangle]
pub unsafe extern "C" fn posix_spawnp() {
    unimplemented!("posix_spawnp");
}

#[no_mangle]
pub unsafe extern "C" fn posix_spawnattr_destroy() {
    unimplemented!("posix_spawnattr_destroy")
}

#[no_mangle]
pub unsafe extern "C" fn posix_spawnattr_init() {
    unimplemented!("posix_spawnattr_init")
}

#[no_mangle]
pub unsafe extern "C" fn posix_spawnattr_setflags() {
    unimplemented!("posix_spawnattr_setflags")
}

#[no_mangle]
pub unsafe extern "C" fn posix_spawnattr_setsigdefault() {
    unimplemented!("posix_spawnattr_setsigdefault")
}

#[no_mangle]
pub unsafe extern "C" fn posix_spawnattr_setsigmask() {
    unimplemented!("posix_spawnsetsigmask")
}

#[no_mangle]
pub unsafe extern "C" fn posix_spawn_file_actions_adddup2() {
    unimplemented!("posix_spawn_file_actions_adddup2")
}

#[no_mangle]
pub unsafe extern "C" fn posix_spawn_file_actions_destroy() {
    unimplemented!("posix_spawn_file_actions_destroy")
}

#[no_mangle]
pub unsafe extern "C" fn posix_spawn_file_actions_init() {
    unimplemented!("posix_spawn_file_actions_init")
}

// time

#[no_mangle]
pub unsafe extern "C" fn clock_gettime(id: c_int, tp: *mut rsix::time::Timespec) -> c_int {
    let id = match id {
        data::CLOCK_MONOTONIC => rsix::time::ClockId::Monotonic,
        data::CLOCK_REALTIME => rsix::time::ClockId::Realtime,
        _ => panic!("unimplemented clock({})", id),
    };
    *tp = rsix::time::clock_gettime(id);
    0
}

#[no_mangle]
pub unsafe extern "C" fn nanosleep(
    req: *const rsix::time::Timespec,
    rem: *mut rsix::time::Timespec,
) -> c_int {
    match rsix::time::nanosleep(&*req) {
        rsix::time::NanosleepRelativeResult::Ok => 0,
        rsix::time::NanosleepRelativeResult::Interrupted(remaining) => {
            *rem = remaining;
            *__errno_location() = rsix::io::Error::INTR.raw_os_error();
            -1
        }
        rsix::time::NanosleepRelativeResult::Err(err) => {
            *__errno_location() = err.raw_os_error();
            -1
        }
    }
}

// math

#[no_mangle]
pub unsafe extern "C" fn acos(x: f64) -> f64 {
    libm::acos(x)
}
#[no_mangle]
pub unsafe extern "C" fn acosf(x: f32) -> f32 {
    libm::acosf(x)
}
#[no_mangle]
pub unsafe extern "C" fn acosh(x: f64) -> f64 {
    libm::acosh(x)
}
#[no_mangle]
pub unsafe extern "C" fn acoshf(x: f32) -> f32 {
    libm::acoshf(x)
}
#[no_mangle]
pub unsafe extern "C" fn asin(x: f64) -> f64 {
    libm::asin(x)
}
#[no_mangle]
pub unsafe extern "C" fn asinf(x: f32) -> f32 {
    libm::asinf(x)
}
#[no_mangle]
pub unsafe extern "C" fn asinh(x: f64) -> f64 {
    libm::asinh(x)
}
#[no_mangle]
pub unsafe extern "C" fn asinhf(x: f32) -> f32 {
    libm::asinhf(x)
}
#[no_mangle]
pub unsafe extern "C" fn atan(x: f64) -> f64 {
    libm::atan(x)
}
#[no_mangle]
pub unsafe extern "C" fn atan2(x: f64, y: f64) -> f64 {
    libm::atan2(x, y)
}
#[no_mangle]
pub unsafe extern "C" fn atan2f(x: f32, y: f32) -> f32 {
    libm::atan2f(x, y)
}
#[no_mangle]
pub unsafe extern "C" fn atanf(x: f32) -> f32 {
    libm::atanf(x)
}
#[no_mangle]
pub unsafe extern "C" fn atanh(x: f64) -> f64 {
    libm::atanh(x)
}
#[no_mangle]
pub unsafe extern "C" fn atanhf(x: f32) -> f32 {
    libm::atanhf(x)
}
#[no_mangle]
pub unsafe extern "C" fn cbrt(x: f64) -> f64 {
    libm::cbrt(x)
}
#[no_mangle]
pub unsafe extern "C" fn cbrtf(x: f32) -> f32 {
    libm::cbrtf(x)
}
#[no_mangle]
pub unsafe extern "C" fn ceil(x: f64) -> f64 {
    libm::ceil(x)
}
#[no_mangle]
pub unsafe extern "C" fn ceilf(x: f32) -> f32 {
    libm::ceilf(x)
}
#[no_mangle]
pub unsafe extern "C" fn copysign(x: f64, y: f64) -> f64 {
    libm::copysign(x, y)
}
#[no_mangle]
pub unsafe extern "C" fn copysignf(x: f32, y: f32) -> f32 {
    libm::copysignf(x, y)
}
#[no_mangle]
pub unsafe extern "C" fn cos(x: f64) -> f64 {
    libm::cos(x)
}
#[no_mangle]
pub unsafe extern "C" fn cosf(x: f32) -> f32 {
    libm::cosf(x)
}
#[no_mangle]
pub unsafe extern "C" fn cosh(x: f64) -> f64 {
    libm::cosh(x)
}
#[no_mangle]
pub unsafe extern "C" fn coshf(x: f32) -> f32 {
    libm::coshf(x)
}
#[no_mangle]
pub unsafe extern "C" fn erf(x: f64) -> f64 {
    libm::erf(x)
}
#[no_mangle]
pub unsafe extern "C" fn erfc(x: f64) -> f64 {
    libm::erfc(x)
}
#[no_mangle]
pub unsafe extern "C" fn erfcf(x: f32) -> f32 {
    libm::erfcf(x)
}
#[no_mangle]
pub unsafe extern "C" fn erff(x: f32) -> f32 {
    libm::erff(x)
}
#[no_mangle]
pub unsafe extern "C" fn exp(x: f64) -> f64 {
    libm::exp(x)
}
#[no_mangle]
pub unsafe extern "C" fn exp2(x: f64) -> f64 {
    libm::exp2(x)
}
#[no_mangle]
pub unsafe extern "C" fn exp2f(x: f32) -> f32 {
    libm::exp2f(x)
}
#[no_mangle]
pub unsafe extern "C" fn exp10(x: f64) -> f64 {
    libm::exp10(x)
}
#[no_mangle]
pub unsafe extern "C" fn exp10f(x: f32) -> f32 {
    libm::exp10f(x)
}
#[no_mangle]
pub unsafe extern "C" fn expf(x: f32) -> f32 {
    libm::expf(x)
}
#[no_mangle]
pub unsafe extern "C" fn expm1(x: f64) -> f64 {
    libm::expm1(x)
}
#[no_mangle]
pub unsafe extern "C" fn expm1f(x: f32) -> f32 {
    libm::expm1f(x)
}
#[no_mangle]
pub unsafe extern "C" fn fabs(x: f64) -> f64 {
    libm::fabs(x)
}
#[no_mangle]
pub unsafe extern "C" fn fabsf(x: f32) -> f32 {
    libm::fabsf(x)
}
#[no_mangle]
pub unsafe extern "C" fn fdim(x: f64, y: f64) -> f64 {
    libm::fdim(x, y)
}
#[no_mangle]
pub unsafe extern "C" fn fdimf(x: f32, y: f32) -> f32 {
    libm::fdimf(x, y)
}
#[no_mangle]
pub unsafe extern "C" fn floor(x: f64) -> f64 {
    libm::floor(x)
}
#[no_mangle]
pub unsafe extern "C" fn floorf(x: f32) -> f32 {
    libm::floorf(x)
}
#[no_mangle]
pub unsafe extern "C" fn fma(x: f64, y: f64, z: f64) -> f64 {
    libm::fma(x, y, z)
}
#[no_mangle]
pub unsafe extern "C" fn fmaf(x: f32, y: f32, z: f32) -> f32 {
    libm::fmaf(x, y, z)
}
#[no_mangle]
pub unsafe extern "C" fn fmax(x: f64, y: f64) -> f64 {
    libm::fmax(x, y)
}
#[no_mangle]
pub unsafe extern "C" fn fmaxf(x: f32, y: f32) -> f32 {
    libm::fmaxf(x, y)
}
#[no_mangle]
pub unsafe extern "C" fn fmin(x: f64, y: f64) -> f64 {
    libm::fmin(x, y)
}
#[no_mangle]
pub unsafe extern "C" fn fminf(x: f32, y: f32) -> f32 {
    libm::fminf(x, y)
}
#[no_mangle]
pub unsafe extern "C" fn fmod(x: f64, y: f64) -> f64 {
    libm::fmod(x, y)
}
#[no_mangle]
pub unsafe extern "C" fn fmodf(x: f32, y: f32) -> f32 {
    libm::fmodf(x, y)
}
#[no_mangle]
pub unsafe extern "C" fn frexp(x: f64, y: *mut i32) -> f64 {
    let (a, b) = libm::frexp(x);
    *y = b;
    a
}
#[no_mangle]
pub unsafe extern "C" fn frexpf(x: f32, y: *mut i32) -> f32 {
    let (a, b) = libm::frexpf(x);
    *y = b;
    a
}
#[no_mangle]
pub unsafe extern "C" fn hypot(x: f64, y: f64) -> f64 {
    libm::hypot(x, y)
}
#[no_mangle]
pub unsafe extern "C" fn hypotf(x: f32, y: f32) -> f32 {
    libm::hypotf(x, y)
}
#[no_mangle]
pub unsafe extern "C" fn ilogb(x: f64) -> i32 {
    libm::ilogb(x)
}
#[no_mangle]
pub unsafe extern "C" fn ilogbf(x: f32) -> i32 {
    libm::ilogbf(x)
}
#[no_mangle]
pub unsafe extern "C" fn j0(x: f64) -> f64 {
    libm::j0(x)
}
#[no_mangle]
pub unsafe extern "C" fn j0f(x: f32) -> f32 {
    libm::j0f(x)
}
#[no_mangle]
pub unsafe extern "C" fn j1(x: f64) -> f64 {
    libm::j1(x)
}
#[no_mangle]
pub unsafe extern "C" fn j1f(x: f32) -> f32 {
    libm::j1f(x)
}
#[no_mangle]
pub unsafe extern "C" fn jn(x: i32, y: f64) -> f64 {
    libm::jn(x, y)
}
#[no_mangle]
pub unsafe extern "C" fn jnf(x: i32, y: f32) -> f32 {
    libm::jnf(x, y)
}
#[no_mangle]
pub unsafe extern "C" fn ldexp(x: f64, y: i32) -> f64 {
    libm::ldexp(x, y)
}
#[no_mangle]
pub unsafe extern "C" fn ldexpf(x: f32, y: i32) -> f32 {
    libm::ldexpf(x, y)
}
#[no_mangle]
pub unsafe extern "C" fn lgamma_r(x: f64, y: *mut i32) -> f64 {
    let (a, b) = libm::lgamma_r(x);
    *y = b;
    a
}
#[no_mangle]
pub unsafe extern "C" fn lgammaf_r(x: f32, y: *mut i32) -> f32 {
    let (a, b) = libm::lgammaf_r(x);
    *y = b;
    a
}
#[no_mangle]
pub unsafe extern "C" fn log(x: f64) -> f64 {
    libm::log(x)
}
#[no_mangle]
pub unsafe extern "C" fn log1p(x: f64) -> f64 {
    libm::log1p(x)
}
#[no_mangle]
pub unsafe extern "C" fn log1pf(x: f32) -> f32 {
    libm::log1pf(x)
}
#[no_mangle]
pub unsafe extern "C" fn log2(x: f64) -> f64 {
    libm::log2(x)
}
#[no_mangle]
pub unsafe extern "C" fn log2f(x: f32) -> f32 {
    libm::log2f(x)
}
#[no_mangle]
pub unsafe extern "C" fn log10(x: f64) -> f64 {
    libm::log10(x)
}
#[no_mangle]
pub unsafe extern "C" fn log10f(x: f32) -> f32 {
    libm::log10f(x)
}
#[no_mangle]
pub unsafe extern "C" fn logf(x: f32) -> f32 {
    libm::logf(x)
}
#[no_mangle]
pub unsafe extern "C" fn modf(x: f64, y: *mut f64) -> f64 {
    let (a, b) = libm::modf(x);
    *y = b;
    a
}
#[no_mangle]
pub unsafe extern "C" fn modff(x: f32, y: *mut f32) -> f32 {
    let (a, b) = libm::modff(x);
    *y = b;
    a
}
#[no_mangle]
pub unsafe extern "C" fn nextafter(x: f64, y: f64) -> f64 {
    libm::nextafter(x, y)
}
#[no_mangle]
pub unsafe extern "C" fn nextafterf(x: f32, y: f32) -> f32 {
    libm::nextafterf(x, y)
}
#[no_mangle]
pub unsafe extern "C" fn pow(x: f64, y: f64) -> f64 {
    libm::pow(x, y)
}
#[no_mangle]
pub unsafe extern "C" fn powf(x: f32, y: f32) -> f32 {
    libm::powf(x, y)
}
#[no_mangle]
pub unsafe extern "C" fn remainder(x: f64, y: f64) -> f64 {
    libm::remainder(x, y)
}
#[no_mangle]
pub unsafe extern "C" fn remainderf(x: f32, y: f32) -> f32 {
    libm::remainderf(x, y)
}
#[no_mangle]
pub unsafe extern "C" fn remquo(x: f64, y: f64, z: *mut i32) -> f64 {
    let (a, b) = libm::remquo(x, y);
    *z = b;
    a
}
#[no_mangle]
pub unsafe extern "C" fn remquof(x: f32, y: f32, z: *mut i32) -> f32 {
    let (a, b) = libm::remquof(x, y);
    *z = b;
    a
}
#[no_mangle]
pub unsafe extern "C" fn round(x: f64) -> f64 {
    libm::round(x)
}
#[no_mangle]
pub unsafe extern "C" fn roundf(x: f32) -> f32 {
    libm::roundf(x)
}
#[no_mangle]
pub unsafe extern "C" fn scalbn(x: f64, y: i32) -> f64 {
    libm::scalbn(x, y)
}
#[no_mangle]
pub unsafe extern "C" fn scalbnf(x: f32, y: i32) -> f32 {
    libm::scalbnf(x, y)
}
#[no_mangle]
pub unsafe extern "C" fn sin(x: f64) -> f64 {
    libm::sin(x)
}
#[no_mangle]
pub unsafe extern "C" fn sincos(x: f64, y: *mut f64, z: *mut f64) {
    let (a, b) = libm::sincos(x);
    *y = a;
    *z = b;
}
#[no_mangle]
pub unsafe extern "C" fn sincosf(x: f32, y: *mut f32, z: *mut f32) {
    let (a, b) = libm::sincosf(x);
    *y = a;
    *z = b;
}
#[no_mangle]
pub unsafe extern "C" fn sinf(x: f32) -> f32 {
    libm::sinf(x)
}
#[no_mangle]
pub unsafe extern "C" fn sinh(x: f64) -> f64 {
    libm::sinh(x)
}
#[no_mangle]
pub unsafe extern "C" fn sinhf(x: f32) -> f32 {
    libm::sinhf(x)
}
#[no_mangle]
pub unsafe extern "C" fn sqrt(x: f64) -> f64 {
    libm::sqrt(x)
}
#[no_mangle]
pub unsafe extern "C" fn sqrtf(x: f32) -> f32 {
    libm::sqrtf(x)
}
#[no_mangle]
pub unsafe extern "C" fn tan(x: f64) -> f64 {
    libm::tan(x)
}
#[no_mangle]
pub unsafe extern "C" fn tanf(x: f32) -> f32 {
    libm::tanf(x)
}
#[no_mangle]
pub unsafe extern "C" fn tanh(x: f64) -> f64 {
    libm::tanh(x)
}
#[no_mangle]
pub unsafe extern "C" fn tanhf(x: f32) -> f32 {
    libm::tanhf(x)
}
#[no_mangle]
pub unsafe extern "C" fn tgamma(x: f64) -> f64 {
    libm::tgamma(x)
}
#[no_mangle]
pub unsafe extern "C" fn tgammaf(x: f32) -> f32 {
    libm::tgammaf(x)
}
#[no_mangle]
pub unsafe extern "C" fn trunc(x: f64) -> f64 {
    libm::trunc(x)
}
#[no_mangle]
pub unsafe extern "C" fn truncf(x: f32) -> f32 {
    libm::truncf(x)
}
#[no_mangle]
pub unsafe extern "C" fn y0(x: f64) -> f64 {
    libm::y0(x)
}
#[no_mangle]
pub unsafe extern "C" fn y0f(x: f32) -> f32 {
    libm::y0f(x)
}
#[no_mangle]
pub unsafe extern "C" fn y1(x: f64) -> f64 {
    libm::y1(x)
}
#[no_mangle]
pub unsafe extern "C" fn y1f(x: f32) -> f32 {
    libm::y1f(x)
}
#[no_mangle]
pub unsafe extern "C" fn yn(x: i32, y: f64) -> f64 {
    libm::yn(x, y)
}
#[no_mangle]
pub unsafe extern "C" fn ynf(x: i32, y: f32) -> f32 {
    libm::ynf(x, y)
}

// exec

#[no_mangle]
pub unsafe extern "C" fn execvp() {
    unimplemented!("execvp")
}

#[no_mangle]
pub unsafe extern "C" fn fork() {
    unimplemented!("fork")
}

// <https://refspecs.linuxbase.org/LSB_5.0.0/LSB-Core-generic/LSB-Core-generic/baselib---register-atfork.html>
#[no_mangle]
pub unsafe extern "C" fn __register_atfork(
    _prepare: unsafe extern "C" fn(),
    _parent: unsafe extern "C" fn(),
    _child: unsafe extern "C" fn(),
    _dso_handle: *mut c_void,
) -> c_int {
    // We don't implement `fork` yet, so there's nothing to do.
    0
}

#[no_mangle]
pub unsafe extern "C" fn clone3() {
    unimplemented!("clone3")
}

#[no_mangle]
pub unsafe extern "C" fn waitpid() {
    unimplemented!("waitpid")
}

// setjmp

#[no_mangle]
pub unsafe extern "C" fn siglongjmp() {
    unimplemented!("siglongjmp")
}

#[no_mangle]
pub unsafe extern "C" fn __sigsetjmp() {
    unimplemented!("__sigsetjmp")
}

// utilities

fn set_errno<T>(result: Result<T, rsix::io::Error>) -> Option<T> {
    result
        .map_err(|err| unsafe {
            *__errno_location() = err.raw_os_error();
        })
        .ok()
}
