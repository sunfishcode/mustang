//! Selected libc-compatible interfaces.
//!
//! The goal here isn't necessarily to build a complete libc; it's primarily
//! to provide things that `std` and possibly popular crates are currently
//! using.

use rsix::fs::{cwd, openat, Mode, OFlags};
#[cfg(debug_assertions)]
use rsix::io::stderr;
use rsix::io::{MapFlags, MprotectFlags, PipeFlags, ProtFlags};
use rsix::io_lifetimes::{BorrowedFd, OwnedFd};
use std::cmp::Ordering;
use std::ffi::{c_void, CStr};
use std::os::raw::{c_char, c_int, c_long, c_uint, c_ulong};
#[cfg(debug_assertions)]
use std::os::unix::io::AsRawFd;
use std::os::unix::io::{FromRawFd, IntoRawFd};
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
pub unsafe extern "C" fn __xpg_strerror_r() {
    unimplemented!("__xpg_strerror_r")
}

// fs

#[no_mangle]
pub unsafe extern "C" fn open64(pathname: *const c_char, flags: c_int, mode: c_int) -> c_int {
    let flags = OFlags::from_bits(flags as _).unwrap();
    let mode = Mode::from_bits(mode as _).unwrap();
    match set_errno(openat(&cwd(), CStr::from_ptr(pathname), flags, mode)) {
        Some(fd) => {
            let fd: OwnedFd = fd.into();
            fd.into_raw_fd()
        }
        None => return -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn open() {
    unimplemented!("open")
}

#[no_mangle]
pub unsafe extern "C" fn readlink() {
    unimplemented!("readlink")
}

#[no_mangle]
pub unsafe extern "C" fn stat64() {
    unimplemented!("stat64")
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

    let flags = rsix::fs::AtFlags::from_bits(flags as _).unwrap();
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
pub unsafe extern "C" fn realpath() {
    unimplemented!("realpath")
}

#[no_mangle]
pub unsafe extern "C" fn fcntl() {
    unimplemented!("fcntl")
}

#[no_mangle]
pub unsafe extern "C" fn mkdir() {
    unimplemented!("mkdir")
}

#[no_mangle]
pub unsafe extern "C" fn fdatasync() {
    unimplemented!("fdatasync")
}

#[no_mangle]
pub unsafe extern "C" fn fstatat64() {
    unimplemented!("fstatat64")
}

#[no_mangle]
pub unsafe extern "C" fn fsync() {
    unimplemented!("fsync")
}

#[no_mangle]
pub unsafe extern "C" fn ftruncate64() {
    unimplemented!("ftruncate64")
}

#[no_mangle]
pub unsafe extern "C" fn rename() {
    unimplemented!("rename")
}

#[no_mangle]
pub unsafe extern "C" fn rmdir() {
    unimplemented!("rmdir")
}

#[no_mangle]
pub unsafe extern "C" fn unlink() {
    unimplemented!("unlink")
}

#[no_mangle]
pub unsafe extern "C" fn lseek64() {
    unimplemented!("lseek64")
}

#[no_mangle]
pub unsafe extern "C" fn lstat64() {
    unimplemented!("lstat64")
}

#[no_mangle]
pub unsafe extern "C" fn opendir() {
    unimplemented!("opendir")
}

#[no_mangle]
pub unsafe extern "C" fn readdir64_r() {
    unimplemented!("readdir64_r")
}

#[no_mangle]
pub unsafe extern "C" fn closedir() {
    unimplemented!("closedir")
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
pub unsafe extern "C" fn writev() {
    unimplemented!("writev")
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
pub unsafe extern "C" fn readv() {
    unimplemented!("readv")
}

#[no_mangle]
pub unsafe extern "C" fn pread64() {
    unimplemented!("pread64")
}

#[no_mangle]
pub unsafe extern "C" fn poll(_fds: c_void, _nfds: c_ulong, _timeout: c_int) -> c_int {
    // Somehow we timed out before anything happened. Huh.
    0
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

#[cfg(any(target_os = "android", target_os = "linux"))]
const TCGETS: c_long = 0x5401;

#[no_mangle]
pub unsafe extern "C" fn ioctl(fd: c_int, request: c_long, mut args: ...) -> c_int {
    let fd = BorrowedFd::borrow_raw_fd(fd);
    match request {
        TCGETS => match set_errno(rsix::io::ioctl_tcgets(&fd)) {
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
            let a: OwnedFd = a.into();
            *pipefd = a.into_raw_fd();
            let b: OwnedFd = b.into();
            *pipefd.add(1) = b.into_raw_fd();
            0
        }
        None => -1,
    }
}

// malloc

// Large enough for any C type, including v128.
const MALLOC_ALIGN: usize = 16;

#[no_mangle]
pub unsafe extern "C" fn malloc(size: usize) -> *mut c_void {
    let layout = std::alloc::Layout::from_size_align(size, MALLOC_ALIGN).unwrap();
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
    while *s != 0 {
        len += 1;
        s = s.add(1);
    }
    len
}

// mmap

#[cfg(any(target_os = "android", target_os = "linux"))]
const MAP_ANONYMOUS: i32 = 32;

const MAP_ERR: *mut c_void = -1_isize as usize as *mut c_void;

#[no_mangle]
pub unsafe extern "C" fn mmap(
    addr: *mut c_void,
    length: usize,
    prot: c_int,
    flags: c_int,
    fd: c_int,
    offset: i64,
) -> *mut c_void {
    let anon = flags & MAP_ANONYMOUS == MAP_ANONYMOUS;
    let prot = ProtFlags::from_bits(prot as _).unwrap();
    let flags = MapFlags::from_bits((flags & !MAP_ANONYMOUS) as _).unwrap();
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
        None => MAP_ERR,
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

// process

#[no_mangle]
pub unsafe extern "C" fn getpid() {
    unimplemented!("getpid")
}

#[cfg(any(target_os = "android", target_os = "linux"))]
const _SC_PAGESIZE: c_int = 30;

#[cfg(any(target_os = "android", target_os = "linux"))]
const _SC_NPROCESSORS_ONLN: c_int = 84;

#[no_mangle]
pub unsafe extern "C" fn sysconf(name: c_int) -> c_long {
    match name {
        _SC_PAGESIZE => rsix::process::page_size() as _,
        // Oddly, only ever one processor seems to be online.
        _SC_NPROCESSORS_ONLN => 1 as _,
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
    if handle.is_null() && CStr::from_ptr(symbol).to_bytes() == b"statx" {
        return statx as *mut c_void;
    }
    if handle.is_null() && CStr::from_ptr(symbol).to_bytes() == b"__pthread_get_minstack" {
        // Let's just say we don't support this for now.
        return null_mut();
    }
    if handle.is_null() && CStr::from_ptr(symbol).to_bytes() == b"gnu_get_libc_version" {
        // Let's just say we don't support this for now.
        return null_mut();
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
pub unsafe extern "C" fn getuid() {
    unimplemented!("getuid")
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
pub unsafe extern "C" fn syscall() {
    unimplemented!("syscall")
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
pub unsafe extern "C" fn clock_gettime(_id: c_int, _tp: *mut rsix::time::Timespec) {
    unimplemented!("clock_gettime")
}

#[no_mangle]
pub unsafe extern "C" fn nanosleep() {
    unimplemented!("nanosleep")
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
