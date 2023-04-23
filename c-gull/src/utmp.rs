use core::ffi::CStr;
use core::mem::{size_of, zeroed};
use core::ptr::null_mut;
use core::slice;
use errno::{set_errno, Errno};
use libc::{c_char, c_int, utmpx};
use std::cell::OnceCell;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use std::sync::Mutex;

static LOCK: Mutex<OnceCell<(PathBuf, Option<File>, utmpx)>> = Mutex::new(OnceCell::new());

#[no_mangle]
unsafe extern "C" fn getutxent() -> *mut libc::utmpx {
    libc!(libc::getutxent());

    let mut lock = LOCK.lock().unwrap();

    ensure_open_file(&mut *lock);

    let (_path, file, entry) = lock.get_mut().unwrap();

    match file.as_ref().unwrap().read_exact(unsafe {
        slice::from_raw_parts_mut(entry as *mut utmpx as *mut u8, size_of::<utmpx>())
    }) {
        Ok(()) => entry,
        Err(err) => {
            set_errno(Errno(err.raw_os_error().unwrap_or(libc::EIO)));
            null_mut()
        }
    }
}

#[no_mangle]
unsafe extern "C" fn getutxid(ut: *const libc::utmpx) -> *mut libc::utmpx {
    libc!(libc::getutxid(ut));

    let mut lock = LOCK.lock().unwrap();

    ensure_open_file(&mut *lock);

    let (_path, file, entry) = lock.get_mut().unwrap();

    loop {
        match file.as_ref().unwrap().read_exact(unsafe {
            slice::from_raw_parts_mut(entry as *mut utmpx as *mut u8, size_of::<utmpx>())
        }) {
            Ok(()) => {}
            Err(err) => {
                set_errno(Errno(err.raw_os_error().unwrap_or(libc::EIO)));
                return null_mut();
            }
        }

        match (*ut).ut_type {
            libc::RUN_LVL | libc::BOOT_TIME | libc::NEW_TIME | libc::OLD_TIME => {
                if entry.ut_type == (*ut).ut_type {
                    return entry;
                }
            }
            libc::INIT_PROCESS | libc::LOGIN_PROCESS | libc::USER_PROCESS | libc::DEAD_PROCESS => {
                if entry.ut_id == (*ut).ut_id {
                    return entry;
                }
            }
            _ => {}
        }
    }
}

#[no_mangle]
unsafe extern "C" fn getutxline(ut: *const libc::utmpx) -> *mut libc::utmpx {
    libc!(libc::getutxline(ut));

    let mut lock = LOCK.lock().unwrap();

    ensure_open_file(&mut *lock);

    let (_path, file, entry) = lock.get_mut().unwrap();

    loop {
        match file.as_ref().unwrap().read_exact(unsafe {
            slice::from_raw_parts_mut(entry as *mut utmpx as *mut u8, size_of::<utmpx>())
        }) {
            Ok(()) => {}
            Err(err) => {
                set_errno(Errno(err.raw_os_error().unwrap_or(0)));
                return null_mut();
            }
        }

        match entry.ut_type {
            libc::USER_PROCESS | libc::LOGIN_PROCESS => {
                if entry.ut_line == (*ut).ut_line {
                    return entry;
                }
            }
            _ => {}
        }
    }
}

#[no_mangle]
unsafe extern "C" fn setutxent() {
    libc!(libc::setutxent());

    let mut lock = LOCK.lock().unwrap();

    let (_path, _file, _entry) =
        lock.get_or_init(|| (PathBuf::from("/var/run/utmp"), None, unsafe { zeroed() }));

    match &mut lock.get_mut().unwrap().1 {
        Some(file) => {
            file.seek(SeekFrom::Start(0)).ok();
        }
        None => {}
    }
}

#[no_mangle]
unsafe extern "C" fn endutxent() {
    libc!(libc::endutxent());

    let mut lock = LOCK.lock().unwrap();

    let (_path, _file, _entry) =
        lock.get_or_init(|| (PathBuf::from("/var/run/utmp"), None, unsafe { zeroed() }));

    lock.get_mut().unwrap().1 = None;
}

#[no_mangle]
unsafe extern "C" fn utmpxname(file: *const c_char) -> c_int {
    libc!(libc::utmpxname(file));

    let mut lock = LOCK.lock().unwrap();

    let (_path, _file, _entry) =
        lock.get_or_init(|| (PathBuf::from("/var/run/utmp"), None, unsafe { zeroed() }));

    let tuple = lock.get_mut().unwrap();
    tuple.0 = PathBuf::from(OsStr::from_bytes(CStr::from_ptr(file).to_bytes()));
    tuple.1 = None;

    0
}

fn ensure_open_file(lock: &mut OnceCell<(PathBuf, Option<File>, utmpx)>) {
    let (path, file, _entry) =
        lock.get_or_init(|| (PathBuf::from("/var/run/utmp"), None, unsafe { zeroed() }));

    match file {
        Some(_file) => {}
        None => match File::open(path) {
            Ok(opened) => lock.get_mut().unwrap().1 = Some(opened),
            Err(_) => {}
        },
    }
}
