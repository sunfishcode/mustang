use crate::{convert_res, set_errno, Errno};
use libc::c_int;
use rustix::fd::{BorrowedFd, IntoRawFd};
use rustix::time::{TimerfdClockId, TimerfdFlags, TimerfdTimerFlags};

#[no_mangle]
unsafe extern "C" fn timerfd_create(clockid: c_int, flags: c_int) -> c_int {
    libc!(libc::timerfd_create(clockid, flags));

    let clockid = match clockid {
        libc::CLOCK_REALTIME => TimerfdClockId::Realtime,
        libc::CLOCK_MONOTONIC => TimerfdClockId::Monotonic,
        libc::CLOCK_BOOTTIME => TimerfdClockId::Boottime,
        libc::CLOCK_REALTIME_ALARM => TimerfdClockId::RealtimeAlarm,
        libc::CLOCK_BOOTTIME_ALARM => TimerfdClockId::BoottimeAlarm,
        _ => {
            set_errno(Errno(libc::EINVAL));
            return -1;
        }
    };
    let flags = TimerfdFlags::from_bits(flags.try_into().unwrap()).unwrap();
    match convert_res(rustix::time::timerfd_create(clockid, flags)) {
        Some(fd) => fd.into_raw_fd(),
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn timerfd_settime(
    fd: c_int,
    flags: c_int,
    new_value: *const libc::itimerspec,
    old_value: *mut libc::itimerspec,
) -> c_int {
    libc!(libc::timerfd_settime(fd, flags, new_value, old_value));

    let new_value = new_value.read();
    let new_value = rustix::time::Itimerspec {
        it_interval: rustix::time::Timespec {
            tv_sec: new_value.it_interval.tv_sec.into(),
            tv_nsec: new_value.it_interval.tv_nsec as _,
        },
        it_value: rustix::time::Timespec {
            tv_sec: new_value.it_value.tv_sec.into(),
            tv_nsec: new_value.it_value.tv_nsec as _,
        },
    };
    let flags = TimerfdTimerFlags::from_bits(flags.try_into().unwrap()).unwrap();
    match convert_res(rustix::time::timerfd_settime(
        BorrowedFd::borrow_raw(fd),
        flags,
        &new_value,
    )) {
        Some(value) => {
            if !old_value.is_null() {
                let value = libc::itimerspec {
                    it_interval: libc::timespec {
                        tv_sec: value.it_interval.tv_sec as _,
                        tv_nsec: value.it_interval.tv_nsec as _,
                    },
                    it_value: libc::timespec {
                        tv_sec: value.it_value.tv_sec as _,
                        tv_nsec: value.it_value.tv_nsec as _,
                    },
                };
                old_value.write(value);
            }
            0
        }
        None => -1,
    }
}
