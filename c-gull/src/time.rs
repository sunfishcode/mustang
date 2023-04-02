//! Time and date conversion routines.
//!
//! This code is highly experimental.

use errno::{set_errno, Errno};
use libc::{c_char, c_int, c_long, time_t, tm};
use std::cell::OnceCell;
use std::collections::HashSet;
use std::ffi::CString;
use std::ptr::{self, null_mut};
use std::sync::{Mutex, MutexGuard};
use tz::error::{TzError, TzFileError, TzStringError};
use tz::timezone::TransitionRule;
use tz::{DateTime, LocalTimeType, TimeZone};

// These things aren't really thread-safe, but they're implementing C ABIs and
// the C rule is, it's up to the user to ensure that none of the bad things
// happen.
struct SyncTm(tm);
unsafe impl Sync for SyncTm {}

// `tzname`, `timezone`, and `daylight` are guarded by `TIMEZONE_LOCK`.
struct SyncTzName([*mut c_char; 2]);
unsafe impl Sync for SyncTzName {}

// Hold `TIMEZONE_LOCK` when updating `tzname`, `timezone`, and `daylight`.
static TIMEZONE_LOCK: Mutex<(Option<CString>, Option<CString>)> = Mutex::new((None, None));
#[no_mangle]
static mut tzname: SyncTzName = SyncTzName([null_mut(), null_mut()]);
#[no_mangle]
static mut timezone: c_long = 0;
#[no_mangle]
static mut daylight: c_int = 0;

// Name storage for the `tm_zone` field.
static TIMEZONE_NAMES: Mutex<OnceCell<HashSet<CString>>> = Mutex::new(OnceCell::new());

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

    ptr::write(result, date_time_to_tm(date_time));

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

    let time_zone = match time_zone() {
        Ok(time_zone) => time_zone,
        Err(err) => {
            set_errno(tz_error_to_errno(err));
            return null_mut();
        }
    };

    let local_time_type = match time_zone.find_local_time_type((*time).into()) {
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

    ptr::write(result, date_time_to_tm(date_time));

    result
}

fn tm_to_date_time(tm: &tm, time_zone: &TimeZone) -> Result<DateTime, Errno> {
    let tm_year = (*tm).tm_year;
    let tm_mon: u8 = match (*tm).tm_mon.try_into() {
        Ok(tm_mon) => tm_mon,
        Err(_err) => return Err(Errno(libc::EOVERFLOW)),
    };
    let tm_mday = match (*tm).tm_mday.try_into() {
        Ok(tm_mday) => tm_mday,
        Err(_err) => return Err(Errno(libc::EOVERFLOW)),
    };
    let tm_hour = match (*tm).tm_hour.try_into() {
        Ok(tm_hour) => tm_hour,
        Err(_err) => return Err(Errno(libc::EOVERFLOW)),
    };
    let tm_min = match (*tm).tm_min.try_into() {
        Ok(tm_min) => tm_min,
        Err(_err) => return Err(Errno(libc::EOVERFLOW)),
    };
    let tm_sec = match (*tm).tm_sec.try_into() {
        Ok(tm_sec) => tm_sec,
        Err(_err) => return Err(Errno(libc::EOVERFLOW)),
    };

    let utc;
    let (std_time_type, dst_time_type) = if let Some(extra_rule) = time_zone.as_ref().extra_rule() {
        match extra_rule {
            TransitionRule::Fixed(local_time_type) => (local_time_type, None),
            TransitionRule::Alternate(alternate_time_type) => {
                (alternate_time_type.std(), Some(alternate_time_type.dst()))
            }
        }
    } else {
        utc = LocalTimeType::utc();
        (&utc, None)
    };

    let date_time = match (*tm).tm_isdst {
        tm_isdst if tm_isdst < 0 => {
            match DateTime::find(
                tm_year + 1900,
                tm_mon + 1,
                tm_mday,
                tm_hour,
                tm_min,
                tm_sec,
                0,
                time_zone.as_ref(),
            ) {
                Ok(found) => {
                    match found.unique() {
                        Some(date_time) => date_time,
                        None => {
                            // It's not clear what we should do here, so for
                            // now just be conservative.
                            return Err(Errno(libc::EIO));
                        }
                    }
                }
                Err(_err) => return Err(Errno(libc::EIO)),
            }
        }
        tm_isdst => {
            let time_type = if tm_isdst > 0 {
                dst_time_type.unwrap_or(std_time_type)
            } else {
                std_time_type
            };
            match DateTime::new(
                tm_year + 1900,
                tm_mon + 1,
                tm_mday,
                tm_hour,
                tm_min,
                tm_sec,
                0,
                *time_type,
            ) {
                Ok(date_time) => date_time,
                Err(_err) => return Err(Errno(libc::EIO)),
            }
        }
    };

    Ok(date_time)
}

#[no_mangle]
unsafe extern "C" fn mktime(tm: *mut tm) -> time_t {
    libc!(libc::mktime(tm));

    let mut lock = TIMEZONE_LOCK.lock().unwrap();

    clear_timezone(&mut lock);

    let time_zone = match time_zone() {
        Ok(time_zone) => time_zone,
        Err(err) => {
            set_errno(tz_error_to_errno(err));
            return -1;
        }
    };

    let utc;
    let (std_time_type, dst_time_type) = if let Some(extra_rule) = time_zone.as_ref().extra_rule() {
        match extra_rule {
            TransitionRule::Fixed(local_time_type) => (local_time_type, None),
            TransitionRule::Alternate(alternate_time_type) => {
                (alternate_time_type.std(), Some(alternate_time_type.dst()))
            }
        }
    } else {
        utc = LocalTimeType::utc();
        (&utc, None)
    };

    let date_time = match tm_to_date_time(&*tm, &time_zone) {
        Ok(date_time) => date_time,
        Err(errno) => {
            set_errno(errno);
            return -1;
        }
    };

    let unix_time = match date_time.unix_time().try_into() {
        Ok(unix_time) => unix_time,
        Err(_err) => {
            set_errno(Errno(libc::EOVERFLOW));
            -1
        }
    };

    ptr::write(tm, date_time_to_tm(date_time));

    set_timezone(&mut lock, std_time_type, dst_time_type);

    unix_time
}

fn date_time_to_tm(date_time: DateTime) -> tm {
    let local_time_type = date_time.local_time_type();

    let tm_zone = {
        let mut timezone_names = TIMEZONE_NAMES.lock().unwrap();
        timezone_names.get_or_init(|| HashSet::new());
        let cstr = CString::new(local_time_type.time_zone_designation()).unwrap();
        // TODO: Use `get_or_insert` when `hash_set_entry` is stabilized.
        timezone_names.get_mut().unwrap().insert(cstr.clone());
        timezone_names.get().unwrap().get(&cstr).unwrap().as_ptr()
    };

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
        tm_zone,
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

fn tz_error_to_errno(err: TzError) -> Errno {
    match err {
        TzError::IoError(err) => Errno(err.raw_os_error().unwrap_or(libc::EIO)),
        TzError::TzFileError(TzFileError::IoError(err)) => {
            Errno(err.raw_os_error().unwrap_or(libc::EIO))
        }
        TzError::TzStringError(TzStringError::IoError(err)) => {
            Errno(err.raw_os_error().unwrap_or(libc::EIO))
        }
        _ => Errno(libc::EIO),
    }
}

#[no_mangle]
unsafe extern "C" fn timegm(tm: *mut tm) -> time_t {
    libc!(libc::timegm(tm));

    let time_zone_utc = TimeZone::utc();

    let date_time = match tm_to_date_time(&*tm, &time_zone_utc) {
        Ok(date_time) => date_time,
        Err(errno) => {
            set_errno(errno);
            return -1;
        }
    };

    let unix_time = match date_time.unix_time().try_into() {
        Ok(unix_time) => unix_time,
        Err(_err) => {
            set_errno(Errno(libc::EOVERFLOW));
            -1
        }
    };

    unix_time
}

#[no_mangle]
unsafe extern "C" fn timelocal(tm: *mut tm) -> time_t {
    //libc!(libc::timelocal(tm)); // TODO: upstream

    let time_zone = match time_zone() {
        Ok(time_zone) => time_zone,
        Err(err) => {
            set_errno(tz_error_to_errno(err));
            return -1;
        }
    };

    let date_time = match tm_to_date_time(&*tm, &time_zone) {
        Ok(date_time) => date_time,
        Err(errno) => {
            set_errno(errno);
            return -1;
        }
    };

    let unix_time = match date_time.unix_time().try_into() {
        Ok(unix_time) => unix_time,
        Err(_err) => {
            set_errno(Errno(libc::EOVERFLOW));
            -1
        }
    };

    unix_time
}

#[no_mangle]
unsafe extern "C" fn tzset() {
    //libc!(libc::tzset()); // TODO: upstream

    let mut lock = TIMEZONE_LOCK.lock().unwrap();

    clear_timezone(&mut lock);

    let time_zone = match time_zone() {
        Ok(time_zone) => time_zone,
        Err(_) => return,
    };

    if let Some(extra_rule) = time_zone.as_ref().extra_rule() {
        match extra_rule {
            TransitionRule::Fixed(local_time_type) => {
                set_timezone(&mut lock, local_time_type, None);
            }
            TransitionRule::Alternate(alternate_time_type) => {
                let std = alternate_time_type.std();
                let dst = alternate_time_type.dst();
                set_timezone(&mut lock, std, Some(dst));
            }
        }
    }
}

fn time_zone() -> Result<TimeZone, TzError> {
    match std::env::var("TZ") {
        Ok(tz) => TimeZone::from_posix_tz(&tz),
        Err(_) => TimeZone::local(),
    }
}

unsafe fn set_timezone(
    guard: &mut MutexGuard<(Option<CString>, Option<CString>)>,
    std: &LocalTimeType,
    dst: Option<&LocalTimeType>,
) {
    guard.0 = Some(CString::new(std.time_zone_designation()).unwrap());
    tzname.0[0] = guard.0.as_ref().unwrap().as_ptr() as *mut c_char;

    match dst {
        Some(dst) => {
            guard.1 = Some(CString::new(dst.time_zone_designation()).unwrap());
            tzname.0[1] = guard.1.as_ref().unwrap().as_ptr() as *mut c_char;
            daylight = 1;
        }
        None => {
            guard.1 = None;
            tzname.0[1] = guard.0.as_ref().unwrap().as_ptr() as *mut c_char;
            daylight = 0;
        }
    }

    let ut_offset = std.ut_offset();
    timezone = -c_long::from(ut_offset);
}

unsafe fn clear_timezone(guard: &mut MutexGuard<(Option<CString>, Option<CString>)>) {
    guard.0 = None;
    guard.1 = None;
    tzname.0[0] = null_mut();
    tzname.0[1] = null_mut();
    timezone = 0;
    daylight = 0;
}
