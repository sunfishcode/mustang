use crate::convert_res;
use core::mem::zeroed;
use core::ptr::{self, null};
use errno::{set_errno, Errno};
use libc::{c_char, c_int, c_long, c_void};

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
            ptr::invalid_mut(libc::getrandom(buf, len, flags) as _)
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
        libc::SYS_epoll_create1 => {
            let flags = args.arg::<c_int>();
            ptr::invalid_mut(libc::epoll_create(flags) as isize as usize)
        }
        libc::SYS_timerfd_create => {
            let clockid = args.arg::<c_int>();
            let flags = args.arg::<c_int>();
            ptr::invalid_mut(libc::timerfd_create(clockid, flags) as isize as usize)
        }
        libc::SYS_timerfd_settime => {
            let fd = args.arg::<c_int>();
            let flags = args.arg::<c_int>();
            let new_value = args.arg::<*const libc::itimerspec>();
            let old_value = args.arg::<*mut libc::itimerspec>();
            ptr::invalid_mut(
                libc::timerfd_settime(fd, flags, new_value, old_value) as isize as usize,
            )
        }
        libc::SYS_utimensat => {
            let fd = args.arg::<c_int>();
            let path = args.arg::<*const c_char>();
            let times = args.arg::<*const libc::timespec>();
            let flags = args.arg::<c_int>();
            ptr::invalid_mut(libc::utimensat(fd, path, times, flags) as isize as usize)
        }
        libc::SYS_fdatasync => {
            let fd = args.arg::<c_int>();
            ptr::invalid_mut(libc::fdatasync(fd) as isize as usize)
        }
        libc::SYS_syncfs => {
            let fd = args.arg::<c_int>();
            ptr::invalid_mut(libc::syncfs(fd) as isize as usize)
        }
        _ => unimplemented!("syscall({:?})", number),
    }
}
