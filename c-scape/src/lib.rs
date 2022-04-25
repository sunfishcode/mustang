#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]
#![no_builtins] // don't let LLVM optimize our `memcpy` into a `memcpy` call
#![feature(thread_local)] // for `__errno_location`
#![feature(c_variadic)] // for `ioctl` etc.
#![feature(rustc_private)] // for compiler-builtins
#![feature(untagged_unions)] // for `PthreadMutexT`
#![feature(atomic_mut_ptr)] // for `RawMutex`
#![feature(strict_provenance)]
#![deny(fuzzy_provenance_casts)]
#![deny(lossy_provenance_casts)]
#![feature(try_blocks)]

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
use core::convert::TryInto;
use core::ffi::c_void;
#[cfg(not(target_arch = "riscv64"))]
use core::mem::align_of;
#[cfg(not(target_os = "wasi"))]
use core::mem::{size_of, zeroed};
use core::ptr::{self, null, null_mut};
use core::slice;
use errno::{set_errno, Errno};
use error_str::error_str;
use libc::{c_char, c_int, c_long, c_ulong};
use rustix::ffi::ZStr;
use rustix::fs::{AtFlags, Mode, OFlags};

// fs
mod fs;

// io
mod io;

// mmap
#[cfg(not(target_os = "wasi"))]
mod mmap;

// net
#[cfg(feature = "net")]
mod net;

// process
#[cfg(not(target_os = "wasi"))]
mod process;

// threads
#[cfg(feature = "threads")]
mod threads;

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
            total_ptr.wrapping_add(offset).cast()
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
    tagged_alloc(layout).cast()
}

#[no_mangle]
unsafe extern "C" fn realloc(old: *mut c_void, size: usize) -> *mut c_void {
    libc!(libc::realloc(old, size));

    if old.is_null() {
        malloc(size)
    } else {
        let old_layout = get_layout(old.cast());
        if old_layout.size() >= size {
            return old;
        }

        let new = malloc(size);
        memcpy(new, old, core::cmp::min(size, old_layout.size()));
        tagged_dealloc(old.cast());
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
    let ptr = tagged_alloc(layout).cast();

    *memptr = ptr;
    0
}

#[no_mangle]
unsafe extern "C" fn free(ptr: *mut c_void) {
    libc!(libc::free(ptr));

    if ptr.is_null() {
        return;
    }

    tagged_dealloc(ptr.cast());
}

// mem

#[no_mangle]
unsafe extern "C" fn memcmp(a: *const c_void, b: *const c_void, len: usize) -> c_int {
    libc!(libc::memcmp(a, b, len));

    compiler_builtins::mem::memcmp(a.cast(), b.cast(), len)
}

#[no_mangle]
unsafe extern "C" fn bcmp(a: *const c_void, b: *const c_void, len: usize) -> c_int {
    // `bcmp` is just an alias for `memcmp`.
    libc!(libc::memcmp(a, b, len));

    compiler_builtins::mem::bcmp(a.cast(), b.cast(), len)
}

#[no_mangle]
unsafe extern "C" fn memcpy(dst: *mut c_void, src: *const c_void, len: usize) -> *mut c_void {
    libc!(libc::memcpy(dst, src, len));

    compiler_builtins::mem::memcpy(dst.cast(), src.cast(), len).cast()
}

#[no_mangle]
unsafe extern "C" fn memmove(dst: *mut c_void, src: *const c_void, len: usize) -> *mut c_void {
    libc!(libc::memmove(dst, src, len));

    compiler_builtins::mem::memmove(dst.cast(), src.cast(), len).cast()
}

#[no_mangle]
unsafe extern "C" fn memset(dst: *mut c_void, fill: c_int, len: usize) -> *mut c_void {
    libc!(libc::memset(dst, fill, len));

    compiler_builtins::mem::memset(dst.cast(), fill, len).cast()
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

// `getauxval` usually returns `unsigned long`, but we make it a pointer type
// so that it preserves provenance.
#[no_mangle]
unsafe extern "C" fn getauxval(type_: c_ulong) -> *mut c_void {
    libc!(ptr::from_exposed_addr_mut(libc::getauxval(type_) as _));
    unimplemented!("unrecognized getauxval {}", type_)
}

#[cfg(target_arch = "aarch64")]
#[no_mangle]
unsafe extern "C" fn __getauxval(type_: c_ulong) -> *mut c_void {
    //libc!(ptr::from_exposed_addr(libc::__getauxval(type_) as _));
    match type_ {
        libc::AT_HWCAP => ptr::invalid_mut(rustix::process::linux_hwcap().0),
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

    libc!(libc::dl_iterate_phdr(callback, data));

    let (phdr, phnum) = rustix::runtime::exe_phdrs();
    let mut info = libc::dl_phdr_info {
        dlpi_addr: (&mut __executable_start as *mut c_void).expose_addr() as _,
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
            todo!()
            // return statx as *mut c_void;
        }
        #[cfg(any(target_os = "android", target_os = "linux"))]
        if ZStr::from_ptr(symbol.cast()).to_bytes() == b"getrandom" {
            return getrandom as *mut c_void;
        }
        #[cfg(any(target_os = "android", target_os = "linux"))]
        if ZStr::from_ptr(symbol.cast()).to_bytes() == b"copy_file_range" {
            // return copy_file_range as *mut c_void;
            todo!()
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
    libc!(libc::sched_getaffinity(pid, cpu_set_size, mask.cast()));
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

// In Linux, `prctl`'s arguments are described as `unsigned long`, however we
// use pointer types in order to preserve provenance.
#[cfg(any(target_os = "android", target_os = "linux"))]
#[no_mangle]
unsafe extern "C" fn prctl(
    option: c_int,
    arg2: *mut c_void,
    _arg3: *mut c_void,
    _arg4: *mut c_void,
    _arg5: *mut c_void,
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
    libc!(libc::sigaction(_signum, _act.cast(), _oldact.cast()));

    // No signals for this handler were ever delivered either. What a coincidence.
    0
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn sigaltstack(_ss: *const c_void, _old_ss: *mut c_void) -> c_int {
    libc!(libc::sigaltstack(_ss.cast(), _old_ss.cast()));

    // Somehow no signals were delivered and the stack wasn't ever used. What luck.
    0
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn sigaddset(_sigset: *mut c_void, _signum: c_int) -> c_int {
    libc!(libc::sigaddset(_sigset.cast(), _signum));
    // Signal sets are not used right now.
    0
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn sigemptyset(_sigset: *mut c_void) -> c_int {
    libc!(libc::sigemptyset(_sigset.cast()));
    // Signal sets are not used right now.
    0
}

// syscall

// `syscall` usually returns `long`, but we make it a pointer type so that it
// preserves provenance.
#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn syscall(number: c_long, mut args: ...) -> *mut c_void {
    match number {
        libc::SYS_getrandom => {
            let buf = args.arg::<*mut c_void>();
            let len = args.arg::<usize>();
            let flags = args.arg::<u32>();
            libc!(ptr::invalid_mut(
                libc::syscall(libc::SYS_getrandom, buf, len, flags) as _
            ));
            ptr::invalid_mut(getrandom(buf, len, flags) as _)
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
            libc!(ptr::invalid_mut(libc::syscall(
                libc::SYS_futex,
                uaddr,
                futex_op,
                val,
                timeout,
                uaddr2,
                val3
            ) as _));
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
                Some(result) => ptr::invalid_mut(result as _),
                None => ptr::invalid_mut(!0),
            }
        }
        libc::SYS_clone3 => {
            // ensure std::process uses fork as fallback code on linux
            set_errno(Errno(libc::ENOSYS));
            ptr::invalid_mut(!0)
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
unsafe extern "C" fn posix_spawnattr_setpgroup(_ptr: *mut c_void, _pgroup: c_int) -> c_int {
    //libc!(libc::posix_spawnattr_setpgroup(ptr, pgroup));
    unimplemented!("posix_spawnattr_setpgroup")
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn posix_spawn_file_actions_adddup2() {
    //libc!(libc::posix_spawn_file_actions_adddup2());
    unimplemented!("posix_spawn_file_actions_adddup2")
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn posix_spawn_file_actions_addchdir_np() {
    //libc!(libc::posix_spawn_file_actions_addchdir_np());
    unimplemented!("posix_spawn_file_actions_addchdir_np")
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
    // FIXME(#95) layout of tp doesn't match signature on i686
    // uncomment once it does:
    // libc!(libc::clock_gettime(id, checked_cast!(tp)));
    libc!(libc::clock_gettime(id, tp.cast()));

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
    libc!(libc::nanosleep(checked_cast!(req), checked_cast!(rem)));

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
        let result = rustix::fs::openat(
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
                let fd = rustix::fs::openat(
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
        if key.to_bytes()
            == slice::from_raw_parts(env.cast::<u8>(), c.offset_from(env).try_into().unwrap())
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
    struct SendSync(*mut c_void);
    unsafe impl Send for SendSync {}
    unsafe impl Sync for SendSync {}
    let arg = SendSync(arg);
    origin::at_exit(Box::new(move || func(arg.0)));
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
    b"2.23\0".as_ptr().cast()
}

// utilities

/// Convert a rustix `Result` into an `Option` with the error stored
/// in `errno`.
fn convert_res<T>(result: Result<T, rustix::io::Error>) -> Option<T> {
    result
        .map_err(|err| set_errno(Errno(err.raw_os_error())))
        .ok()
}
