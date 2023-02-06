#![doc = include_str!("../README.md")]
#![no_std]
#![no_builtins] // don't let LLVM optimize our `memcpy` into a `memcpy` call
#![feature(thread_local)] // for `__errno_location`
#![feature(c_variadic)] // for `ioctl` etc.
#![feature(rustc_private)] // for compiler-builtins
#![feature(atomic_mut_ptr)] // for `RawMutex`
#![feature(strict_provenance)]
#![feature(inline_const)]
#![feature(sync_unsafe_cell)]
#![deny(fuzzy_provenance_casts)]
#![deny(lossy_provenance_casts)]
#![feature(try_blocks)]

extern crate alloc;
extern crate compiler_builtins;

// Re-export the libc crate's API. This allows users to depend on the c-scape
// crate in place of libc.
pub use libc::*;

#[macro_use]
mod use_libc;

#[cfg(not(target_os = "wasi"))]
mod at_fork;
mod error_str;
mod sync_ptr;
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
use core::cell::SyncUnsafeCell;
use core::convert::TryInto;
use core::ffi::c_void;
#[cfg(not(target_os = "wasi"))]
use core::mem::{size_of, zeroed};
use core::ptr::{self, copy_nonoverlapping, null, null_mut};
use core::slice;
use errno::{set_errno, Errno};
use error_str::error_str;
#[cfg(target_vendor = "mustang")]
use libc::c_ulong;
use libc::{c_char, c_int, c_long};
use rustix::ffi::CStr;

// ctype
mod ctype;

// env
mod env;

// fs
mod fs;

// io
mod io;

// malloc
#[cfg(target_vendor = "mustang")]
mod malloc;

// math
mod math;

// mem
mod mem;

// mmap
#[cfg(not(target_os = "wasi"))]
mod mmap;

// net
mod net;

// process
#[cfg(not(target_os = "wasi"))]
mod process;

mod rand;
mod rand48;

// threads
#[cfg(feature = "threads")]
#[cfg(target_vendor = "mustang")]
mod threads;

// time
mod time;

// errno

/// Return the address of the thread-local `errno` state.
///
/// This function conforms to the [LSB `__errno_location`] ABI.
///
/// [LSB `__errno_location`]: https://refspecs.linuxbase.org/LSB_3.1.0/LSB-generic/LSB-generic/baselib-errno-location-1.html
#[no_mangle]
unsafe extern "C" fn __errno_location() -> *mut c_int {
    libc!(libc::__errno_location());

    #[cfg_attr(feature = "threads", thread_local)]
    static mut ERRNO: i32 = 0;
    &mut ERRNO
}

#[no_mangle]
unsafe extern "C" fn strerror(errnum: c_int) -> *mut c_char {
    libc!(libc::strerror(errnum));

    static STORAGE: SyncUnsafeCell<[c_char; 256]> = SyncUnsafeCell::new([0; 256]);

    let storage = SyncUnsafeCell::get(&STORAGE);
    __xpg_strerror_r(errnum, (*storage).as_mut_ptr(), (*storage).len());
    (*storage).as_mut_ptr()
}

#[no_mangle]
unsafe extern "C" fn __xpg_strerror_r(errnum: c_int, buf: *mut c_char, buflen: usize) -> c_int {
    libc!(libc::strerror_r(errnum, buf, buflen));

    if buflen == 0 {
        return libc::ERANGE;
    }

    let message = match error_str(rustix::io::Errno::from_raw_os_error(errnum)) {
        Some(s) => s.to_owned(),
        None => format!("Unknown error {}", errnum),
    };

    let min = core::cmp::min(buflen - 1, message.len());
    copy_nonoverlapping(message.as_ptr().cast(), buf, min);
    buf.add(min).write(b'\0' as libc::c_char);
    0
}

// rand

#[cfg(any(target_os = "android", target_os = "linux"))]
#[no_mangle]
unsafe extern "C" fn getrandom(buf: *mut c_void, buflen: usize, flags: u32) -> isize {
    libc!(libc::getrandom(buf, buflen, flags));

    if buflen == 0 {
        return 0;
    }

    let flags = rustix::rand::GetRandomFlags::from_bits(flags & !0x4).unwrap();

    // `slice::from_raw_parts_mut` assumes that the memory is initialized,
    // which our C API here doesn't guarantee. Since rustix currently requires
    // a slice, use a temporary copy.
    let mut tmp = alloc::vec![0u8; buflen];
    match convert_res(rustix::rand::getrandom(&mut tmp, flags)) {
        Some(num) => {
            core::ptr::copy_nonoverlapping(tmp.as_ptr(), buf.cast::<u8>(), buflen);
            num as isize
        }
        None => -1,
    }
}

#[cfg(any(target_os = "android", target_os = "linux"))]
#[no_mangle]
unsafe extern "C" fn getentropy(buf: *mut c_void, buflen: usize) -> i32 {
    libc!(libc::getentropy(buf, buflen));

    if buflen == 0 {
        return 0;
    }

    if buflen >= 256 {
        set_errno(Errno(libc::EIO));
        return -1;
    }

    let flags = rustix::rand::GetRandomFlags::empty();
    let slice = slice::from_raw_parts_mut(buf.cast::<u8>(), buflen);

    let mut filled = 0usize;

    while filled < buflen {
        match rustix::rand::getrandom(&mut slice[filled..], flags) {
            Ok(num) => filled += num,
            Err(rustix::io::Errno::INTR) => {}
            Err(err) => {
                set_errno(Errno(err.raw_os_error()));
                return -1;
            }
        }
    }

    return 0;
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
        libc::_SC_PAGESIZE => rustix::param::page_size() as _,
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
//
// This is not used in non-mustang configurations because libc startup code
// sometimes needs to call `getauxval` before rustix is initialized.
#[cfg(target_vendor = "mustang")]
#[no_mangle]
unsafe extern "C" fn getauxval(type_: c_ulong) -> *mut c_void {
    libc!(ptr::from_exposed_addr_mut(libc::getauxval(type_) as _));
    unimplemented!("unrecognized getauxval {}", type_)
}

// As with `getauxval`, this is not used in non-mustang configurations because
// libc startup code sometimes needs to call `getauxval` before rustix is
// initialized.
#[cfg(target_arch = "aarch64")]
#[cfg(target_vendor = "mustang")]
#[no_mangle]
unsafe extern "C" fn __getauxval(type_: c_ulong) -> *mut c_void {
    //libc!(ptr::from_exposed_addr(libc::__getauxval(type_) as _));
    match type_ {
        libc::AT_HWCAP => ptr::invalid_mut(rustix::param::linux_hwcap().0),
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
        if CStr::from_ptr(symbol.cast()).to_bytes() == b"statx" {
            todo!()
            // return statx as *mut c_void;
        }
        #[cfg(any(target_os = "android", target_os = "linux"))]
        if CStr::from_ptr(symbol.cast()).to_bytes() == b"getrandom" {
            return getrandom as *mut c_void;
        }
        #[cfg(any(target_os = "android", target_os = "linux"))]
        if CStr::from_ptr(symbol.cast()).to_bytes() == b"copy_file_range" {
            // return copy_file_range as *mut c_void;
            todo!()
        }
        #[cfg(any(target_os = "android", target_os = "linux"))]
        if CStr::from_ptr(symbol.cast()).to_bytes() == b"clone3" {
            // Let's just say we don't support this for now.
            return null_mut();
        }
        if CStr::from_ptr(symbol.cast()).to_bytes() == b"__pthread_get_minstack" {
            // Let's just say we don't support this for now.
            return null_mut();
        }
        #[cfg(target_env = "gnu")]
        if CStr::from_ptr(symbol.cast()).to_bytes() == b"gnu_get_libc_version" {
            return gnu_get_libc_version as *mut c_void;
        }
    }
    unimplemented!("dlsym({:?})", CStr::from_ptr(symbol.cast()))
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
            match convert_res(rustix::runtime::set_thread_name(CStr::from_ptr(
                arg2 as *const _,
            ))) {
                Some(()) => 0,
                None => -1,
            }
        }
        libc::PR_GET_PDEATHSIG => match convert_res(rustix::process::parent_process_death_signal())
        {
            Some(signal) => {
                let sig = signal.map(|s| s as u32 as c_int).unwrap_or(0);
                arg2.cast::<c_int>().write(sig);
                0
            }
            None => -1,
        },
        libc::PR_SET_PDEATHSIG => {
            let arg2_i32 =
                match convert_res(i32::try_from(arg2.addr()).map_err(|_| rustix::io::Errno::RANGE))
                {
                    Some(arg2_i32) => arg2_i32,
                    None => return -1,
                };
            // rustix converts any invalid signal to `None`, but only 0 should get mapped to `None`;
            // any other invalid signal is an error
            let sig = if arg2_i32 == 0 {
                None
            } else {
                match convert_res(
                    rustix::process::Signal::from_raw(arg2_i32).ok_or(rustix::io::Errno::RANGE),
                ) {
                    Some(s) => Some(s),
                    None => return -1,
                }
            };
            match convert_res(rustix::process::set_parent_process_death_signal(sig)) {
                Some(()) => 0,
                None => -1,
            }
        }
        libc::PR_GET_DUMPABLE => match convert_res(rustix::process::dumpable_behavior()) {
            Some(dumpable) => dumpable as i32,
            None => -1,
        },
        libc::PR_SET_DUMPABLE => {
            let arg2_i32 =
                match convert_res(i32::try_from(arg2.addr()).map_err(|_| rustix::io::Errno::RANGE))
                {
                    Some(arg2_i32) => arg2_i32,
                    None => return -1,
                };
            let dumpable = match convert_res(rustix::process::DumpableBehavior::try_from(arg2_i32))
            {
                Some(dumpable) => dumpable,
                None => return -1,
            };
            match convert_res(rustix::process::set_dumpable_behavior(dumpable)) {
                Some(()) => 0,
                None => -1,
            }
        }
        _ => unimplemented!("unrecognized prctl op {}", option),
    }
}

#[cfg(target_os = "linux")]
#[no_mangle]
unsafe extern "C" fn pthread_setname_np(
    thread: libc::pthread_t,
    name: *const libc::c_char,
) -> c_int {
    libc!(libc::pthread_setname_np(thread, name));
    match convert_res(rustix::runtime::set_thread_name(CStr::from_ptr(
        name as *const _,
    ))) {
        Some(()) => 0,
        None => -1,
    }
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn raise(_sig: c_int) -> c_int {
    libc!(libc::raise(_sig));
    unimplemented!("raise")
}

// nss

#[cfg(not(feature = "std"))] // Avoid conflicting with c-gull's more complete `getpwuid_r`.
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
unsafe extern "C" fn __stack_chk_fail() {
    unimplemented!("__stack_chk_fail")
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

// exec

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

// exit

/// Register a function to be called when `exit` is called.
///
/// This function conforms to the [LSB `__cxa_atexit`] ABI.
///
/// [LSB `__cxa_atexit`]: https://refspecs.linuxbase.org/LSB_5.0.0/LSB-Core-generic/LSB-Core-generic/baselib---cxa-atexit.html
#[no_mangle]
unsafe extern "C" fn __cxa_atexit(
    func: unsafe extern "C" fn(*mut c_void),
    arg: *mut c_void,
    _dso: *mut c_void,
) -> c_int {
    // Tell Rust it's ok to send `arg` across threads even though it's a
    // `*mut c_void`.
    struct SendSync(*mut c_void);
    unsafe impl Send for SendSync {}
    unsafe impl Sync for SendSync {}
    let arg = SendSync(arg);

    origin::at_exit(Box::new(move || {
        let arg: SendSync = arg;
        func(arg.0);
    }));

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
fn convert_res<T>(result: Result<T, rustix::io::Errno>) -> Option<T> {
    result
        .map_err(|err| set_errno(Errno(err.raw_os_error())))
        .ok()
}
