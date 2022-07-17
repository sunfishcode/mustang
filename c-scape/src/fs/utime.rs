use core::ffi::CStr;
use core::ptr;
use rustix::fd::BorrowedFd;
use rustix::fs::{AtFlags, Timestamps, UTIME_NOW};
use rustix::time::Timespec;

use libc::{c_char, c_int};

use crate::convert_res;
use errno::{set_errno, Errno};

unsafe fn timestamp_from_timespecs(times: *const [libc::timespec; 2]) -> Timestamps {
    let mut timestamps = Timestamps {
        last_access: Timespec {
            tv_sec: 0,
            tv_nsec: UTIME_NOW,
        },
        last_modification: Timespec {
            tv_sec: 0,
            tv_nsec: UTIME_NOW,
        },
    };

    if !times.is_null() {
        timestamps.last_access.tv_sec = (*times)[0].tv_sec.into();
        timestamps.last_access.tv_nsec = (*times)[0].tv_nsec;
        timestamps.last_modification.tv_sec = (*times)[1].tv_sec.into();
        timestamps.last_modification.tv_nsec = (*times)[1].tv_nsec;
    }

    timestamps
}

#[no_mangle]
unsafe extern "C" fn futimens(fd: c_int, times: *const libc::timespec) -> c_int {
    libc!(libc::futimens(fd, times));

    let times = times as *const [libc::timespec; 2];

    match convert_res(rustix::fs::futimens(
        BorrowedFd::borrow_raw(fd),
        &timestamp_from_timespecs(times),
    )) {
        Some(_) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn utimensat(
    fd: c_int,
    path: *const c_char,
    times: *const libc::timespec,
    flag: c_int,
) -> c_int {
    libc!(libc::utimensat(fd, path, times, flag));

    let times = times as *const [libc::timespec; 2];
    let flags = some_or_ret_einval!(AtFlags::from_bits(flag as _));

    match convert_res(rustix::fs::utimensat(
        BorrowedFd::borrow_raw(fd),
        CStr::from_ptr(path),
        &timestamp_from_timespecs(times),
        flags,
    )) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn utimes(path: *const c_char, times: *const libc::timeval) -> c_int {
    libc!(libc::utimes(path, times));

    let mut arr: [libc::timespec; 2] = core::mem::zeroed();

    let times = times as *const [libc::timeval; 2];
    if !times.is_null() {
        for i in 0..2 {
            arr[i].tv_sec = (*times)[i].tv_sec;

            match (*times)[i].tv_usec.checked_mul(1000) {
                Some(t) => arr[i].tv_nsec = t,
                None => {
                    set_errno(Errno(libc::EINVAL));
                    return -1;
                }
            }
        }
    }

    utimensat(
        libc::AT_FDCWD,
        path,
        if times.is_null() {
            ptr::null()
        } else {
            arr.as_ptr()
        },
        0,
    )
}
