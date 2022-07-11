use core::convert::TryInto;

use errno::{set_errno, Errno};
use libc::c_int;

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
