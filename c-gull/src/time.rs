//! Time and date conversion routines.
//!
//! This code is highly experimental and contains some FIXMEs.

use errno::{set_errno, Errno};
use libc::{c_char, time_t, tm};
use std::ptr::{null_mut, write};
use tz::error::{TzError, TzFileError, TzStringError};
use tz::{DateTime, LocalTimeType, TimeZone};

// These things aren't really thread-safe, but they're implementing C ABIs and
// the C rule is, it's up to the user to ensure that none of the bad things
// happen.
struct SyncTzName([*mut c_char; 2]);
unsafe impl Sync for SyncTzName {}

struct SyncTm(tm);
unsafe impl Sync for SyncTm {}

#[no_mangle]
static mut tzname: SyncTzName = SyncTzName([null_mut(), null_mut()]);

#[no_mangle]
unsafe extern "C" fn gmtime(time: *const time_t) -> *mut tm {
    libc!(libc::gmtime(time));

    static mut TM: SyncTm = SyncTm(blank_tm());
    gmtime_r(time, &mut TM.0)
}

#[no_mangle]
unsafe extern "C" fn gmtime_r(time: *const time_t, result: *mut tm) -> *mut tm {
    libc!(libc::gmtime_r(time, result));

    let utc_time_type = LocalTimeType::utc();

    let date_time = match DateTime::from_timespec_and_local((*time).into(), 0, utc_time_type) {
        Ok(date_time) => date_time,
        Err(_err) => {
            set_errno(Errno(libc::EIO));
            return null_mut();
        }
    };

    write(result, date_time_to_tm(date_time, utc_time_type));

    result
}

#[no_mangle]
unsafe extern "C" fn localtime(time: *const time_t) -> *mut tm {
    libc!(libc::localtime(time));

    static mut TM: SyncTm = SyncTm(blank_tm());
    localtime_r(time, &mut TM.0)
}

#[no_mangle]
unsafe extern "C" fn localtime_r(time: *const time_t, result: *mut tm) -> *mut tm {
    libc!(libc::localtime_r(time, result));

    let time_zone_local = match TimeZone::local() {
        Ok(time_zone_local) => time_zone_local,
        Err(err) => {
            set_errno(Errno(tz_error_to_errno(err)));
            return null_mut();
        }
    };

    let local_time_type = match time_zone_local.find_local_time_type((*time).into()) {
        Ok(local_time_type) => local_time_type,
        Err(_err) => {
            set_errno(Errno(libc::EIO));
            return null_mut();
        }
    };

    let date_time = match DateTime::from_timespec_and_local((*time).into(), 0, *local_time_type) {
        Ok(date_time) => date_time,
        Err(_err) => {
            set_errno(Errno(libc::EIO));
            return null_mut();
        }
    };

    write(result, date_time_to_tm(date_time, *local_time_type));

    result
}

#[no_mangle]
unsafe extern "C" fn mktime(tm: *mut tm) -> time_t {
    libc!(libc::mktime(tm));

    let time_zone_local = match TimeZone::local() {
        Ok(time_zone_local) => time_zone_local,
        Err(err) => {
            set_errno(Errno(tz_error_to_errno(err)));
            return -1;
        }
    };

    let current_local_time_type = match time_zone_local.find_current_local_time_type() {
        Ok(local_time_type) => local_time_type,
        Err(_err) => {
            set_errno(Errno(libc::EIO));
            return -1;
        }
    };

    let tm_year = (*tm).tm_year;
    let tm_mon: u8 = match (*tm).tm_mon.try_into() {
        Ok(tm_mon) => tm_mon,
        Err(_err) => {
            set_errno(Errno(libc::EOVERFLOW));
            return -1;
        }
    };
    let tm_mday = match (*tm).tm_mday.try_into() {
        Ok(tm_mday) => tm_mday,
        Err(_err) => {
            set_errno(Errno(libc::EOVERFLOW));
            return -1;
        }
    };
    let tm_hour = match (*tm).tm_hour.try_into() {
        Ok(tm_hour) => tm_hour,
        Err(_err) => {
            set_errno(Errno(libc::EOVERFLOW));
            return -1;
        }
    };
    let tm_min = match (*tm).tm_min.try_into() {
        Ok(tm_min) => tm_min,
        Err(_err) => {
            set_errno(Errno(libc::EOVERFLOW));
            return -1;
        }
    };
    let tm_sec = match (*tm).tm_sec.try_into() {
        Ok(tm_sec) => tm_sec,
        Err(_err) => {
            set_errno(Errno(libc::EOVERFLOW));
            return -1;
        }
    };

    // FIXME: What should we do with tm.tm_isdst?

    // Create a new date time.
    let date_time = match DateTime::new(
        tm_year + 1900,
        tm_mon + 1,
        tm_mday,
        tm_hour,
        tm_min,
        tm_sec,
        0,
        *current_local_time_type,
    ) {
        Ok(date_time) => date_time,
        Err(_err) => {
            set_errno(Errno(libc::EIO));
            return -1;
        }
    };

    let result = date_time.unix_time();

    set_tzname(current_local_time_type);

    write(tm, date_time_to_tm(date_time, *current_local_time_type));

    match result.try_into() {
        Ok(result) => result,
        Err(_err) => {
            set_errno(Errno(libc::EOVERFLOW));
            -1
        }
    }
}

fn date_time_to_tm(date_time: DateTime, local_time_type: LocalTimeType) -> tm {
    tm {
        tm_year: date_time.year() - 1900,
        tm_mon: (date_time.month() - 1).into(),
        tm_mday: date_time.month_day().into(),
        tm_hour: date_time.hour().into(),
        tm_min: date_time.minute().into(),
        tm_sec: date_time.second().into(),
        tm_wday: date_time.week_day().into(),
        tm_yday: date_time.year_day().into(),
        tm_isdst: if local_time_type.is_dst() { 1 } else { 0 },
        tm_gmtoff: local_time_type.ut_offset().into(),
        tm_zone: b"FIXME(c-gull)\0".as_ptr().cast(),
    }
}

const fn blank_tm() -> tm {
    tm {
        tm_year: 0,
        tm_mon: 0,
        tm_mday: 0,
        tm_hour: 0,
        tm_min: 0,
        tm_sec: 0,
        tm_wday: 0,
        tm_yday: 0,
        tm_isdst: -1,
        tm_gmtoff: 0,
        tm_zone: null_mut(),
    }
}

unsafe fn set_tzname(_time_zone_local: &LocalTimeType) {
    tzname.0[0] = "FIXME(c-gull)\0".as_ptr() as _;
    tzname.0[1] = "FIXME(c-gull)\0".as_ptr() as _;
}

fn tz_error_to_errno(err: TzError) -> i32 {
    match err {
        TzError::IoError(err) => err.raw_os_error().unwrap_or(libc::EIO),
        TzError::TzFileError(TzFileError::IoError(err)) => err.raw_os_error().unwrap_or(libc::EIO),
        TzError::TzStringError(TzStringError::IoError(err)) => {
            err.raw_os_error().unwrap_or(libc::EIO)
        }
        _ => libc::EIO,
    }
}

#[no_mangle]
unsafe extern "C" fn timegm(tm: *mut tm) -> time_t {
    libc!(libc::timegm(tm));

    unimplemented!("timegm")
}
