//! Selected libc-compatible interfaces.
//!
//! The goal here isn't necessarily to build a complete libc; it's primarily
//! to provide things that `std` and possibly popular crates are currently
//! using.

use rsix::fs::{cwd, openat, Mode, OFlags};
#[cfg(debug_assertions)]
use rsix::io::stderr;
use rsix::io::{MapFlags, MprotectFlags, ProtFlags};
use rsix::io_lifetimes::{BorrowedFd, OwnedFd};
use std::cmp::Ordering;
use std::ffi::{c_void, CStr};
use std::os::raw::{c_char, c_int, c_long, c_uint, c_ulong};
#[cfg(debug_assertions)]
use std::os::unix::io::AsRawFd;
use std::os::unix::io::IntoRawFd;
use std::ptr::{null, null_mut};
use std::slice;

// errno

#[no_mangle]
pub unsafe extern "C" fn __errno_location() -> *mut c_int {
    unimplemented!("__errno_location")
}

#[no_mangle]
pub unsafe extern "C" fn __xpg_strerror_r() {
    unimplemented!("__xpg_strerror_r")
}

// fs

#[no_mangle]
pub unsafe extern "C" fn open64(pathname: *const c_char, flags: c_int, mode: c_int) -> c_int {
    let fd: OwnedFd = openat(
        &cwd(),
        CStr::from_ptr(pathname),
        OFlags::from_bits(flags as _).unwrap(),
        Mode::from_bits(mode as _).unwrap(),
    )
    .unwrap()
    .into();
    fd.into_raw_fd()
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
    let r = rsix::fs::fstat(&BorrowedFd::borrow_raw_fd(fd)).unwrap();
    *stat = r;
    0
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
        (*__errno_location()) = rsix::io::Error::FAULT.raw_os_error();
        return -1;
    }

    let r = rsix::fs::statx(
        &BorrowedFd::borrow_raw_fd(dirfd),
        CStr::from_ptr(path),
        rsix::fs::AtFlags::from_bits(flags as _).unwrap(),
        rsix::fs::StatxFlags::from_bits(mask).unwrap(),
    )
    .unwrap();
    *stat = r;
    0
}

#[no_mangle]
pub unsafe extern "C" fn realpath() {
    unimplemented!("realpath")
}

#[no_mangle]
pub unsafe extern "C" fn fcntl() {
    unimplemented!("fcntl")
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

    rsix::io::write(
        &BorrowedFd::borrow_raw_fd(fd),
        slice::from_raw_parts(ptr.cast::<u8>(), len),
    )
    .unwrap() as isize
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

    rsix::io::read(
        &BorrowedFd::borrow_raw_fd(fd),
        slice::from_raw_parts_mut(ptr.cast::<u8>(), len),
    )
    .unwrap() as isize
}

#[no_mangle]
pub unsafe extern "C" fn readv() {
    unimplemented!("readv")
}

#[no_mangle]
pub unsafe extern "C" fn poll(_fds: c_void, _nfds: c_ulong, _timeout: c_int) -> c_int {
    // Somehow we timed out before anything happened. Huh.
    0
}

#[no_mangle]
pub unsafe extern "C" fn close(_fd: c_int) -> c_int {
    unimplemented!("close")
}

#[no_mangle]
pub unsafe extern "C" fn dup2() {
    unimplemented!("dup2")
}

#[no_mangle]
pub unsafe extern "C" fn isatty() {
    unimplemented!("isatty")
}

#[no_mangle]
pub unsafe extern "C" fn ioctl() {
    unimplemented!("ioctl")
}

#[no_mangle]
pub unsafe extern "C" fn pipe2() {
    unimplemented!("pipe2")
}

// malloc

#[no_mangle]
pub unsafe extern "C" fn malloc(_size: usize) -> *mut c_void {
    unimplemented!("malloc")
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
    let product = nmemb.checked_mul(size).unwrap();
    let ptr = malloc(product);
    memset(ptr, 0, product)
}

#[no_mangle]
pub unsafe extern "C" fn posix_memalign(
    _memptr: *mut *mut c_void,
    _alignment: usize,
    _size: usize,
) -> c_int {
    unimplemented!("posix_memalign")
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

#[no_mangle]
pub unsafe extern "C" fn mmap(
    addr: *mut c_void,
    length: usize,
    prot: c_int,
    flags: c_int,
    fd: c_int,
    offset: i64,
) -> *mut c_void {
    if flags & MAP_ANONYMOUS == MAP_ANONYMOUS {
        let flags = flags & !MAP_ANONYMOUS;
        rsix::io::mmap_anonymous(
            addr,
            length,
            ProtFlags::from_bits(prot as _).unwrap(),
            MapFlags::from_bits(flags as _).unwrap(),
        )
        .unwrap()
    } else {
        rsix::io::mmap(
            addr,
            length,
            ProtFlags::from_bits(prot as _).unwrap(),
            MapFlags::from_bits(flags as _).unwrap(),
            &BorrowedFd::borrow_raw_fd(fd),
            offset as _,
        )
        .unwrap()
    }
}

#[no_mangle]
pub unsafe extern "C" fn munmap(ptr: *mut c_void, len: usize) -> c_int {
    rsix::io::munmap(ptr, len).unwrap();
    0
}

#[no_mangle]
pub unsafe extern "C" fn mprotect(addr: *mut c_void, length: usize, prot: c_int) -> c_int {
    rsix::io::mprotect(addr, length, MprotectFlags::from_bits(prot as _).unwrap()).unwrap();
    0
}

// process

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

// math

#[no_mangle]
pub unsafe extern "C" fn floor(x: f64) -> f64 {
    libm::floor(x)
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
