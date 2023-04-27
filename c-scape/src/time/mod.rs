use core::convert::TryInto;

use errno::{set_errno, Errno};
use libc::c_int;

use crate::convert_res;

fn rustix_timespec_to_libc_timespec(
    rustix_time: rustix::time::Timespec,
) -> Result<libc::timespec, core::num::TryFromIntError> {
    // SAFETY: libc structs can be zero-initalized freely
    let mut time: libc::timespec = unsafe { core::mem::zeroed() };
    time.tv_sec = rustix_time.tv_sec.try_into()?;
    time.tv_nsec = rustix_time.tv_nsec.try_into()?;
    Ok(time)
}

#[no_mangle]
unsafe extern "C" fn clock_gettime(id: c_int, tp: *mut libc::timespec) -> c_int {
    libc!(libc::clock_gettime(id, tp));

    let id = match id {
        libc::CLOCK_MONOTONIC => rustix::time::ClockId::Monotonic,
        libc::CLOCK_REALTIME => rustix::time::ClockId::Realtime,
        _ => panic!("unimplemented clock({})", id),
    };

    match rustix_timespec_to_libc_timespec(rustix::time::clock_gettime(id)) {
        Ok(t) => {
            *tp = t;
            0
        }
        Err(_) => {
            set_errno(Errno(libc::EOVERFLOW));
            -1
        }
    }
}

#[no_mangle]
unsafe extern "C" fn time(t: *mut libc::time_t) -> libc::time_t {
    libc!(libc::time(t));

    let mut ts: libc::timespec = { core::mem::zeroed() };
    if clock_gettime(libc::CLOCK_REALTIME, &mut ts) == -1 {
        return -1;
    }

    if !t.is_null() {
        *t = ts.tv_sec;
    }

    ts.tv_sec
}

#[no_mangle]
unsafe extern "C" fn gettimeofday(t: *mut libc::timeval, _tz: *mut libc::timezone) -> c_int {
    libc!(libc::gettimeofday(t, _tz));

    if t.is_null() {
        return 0;
    }

    let mut ts: libc::timespec = { core::mem::zeroed() };
    if clock_gettime(libc::CLOCK_REALTIME, &mut ts) == -1 {
        return -1;
    }

    if !t.is_null() {
        (*t).tv_sec = ts.tv_sec;
        (*t).tv_usec = ts.tv_nsec / 1000;
    }

    0
}

#[no_mangle]
unsafe extern "C" fn nanosleep(req: *const libc::timespec, rem: *mut libc::timespec) -> c_int {
    libc!(libc::nanosleep(req, rem));

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

#[no_mangle]
unsafe extern "C" fn clock_settime(id: c_int, tp: *mut libc::timespec) -> c_int {
    libc!(libc::clock_settime(id, tp));

    let id = match id {
        libc::CLOCK_MONOTONIC => rustix::time::ClockId::Monotonic,
        libc::CLOCK_REALTIME => rustix::time::ClockId::Realtime,
        _ => panic!("unimplemented clock({})", id),
    };

    let timespec = rustix::time::Timespec {
        tv_sec: (*tp).tv_sec.into(),
        tv_nsec: (*tp).tv_nsec as _,
    };

    match convert_res(rustix::time::clock_settime(id, timespec)) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn difftime(time1: libc::time_t, time0: libc::time_t) -> f64 {
    libc!(libc::difftime(time1, time0));

    (time1 as i128 - time0 as i128) as f64
}
