//! Name Service Switch functions.
//!
//! In order to avoid implementing `dlopen`, while still correctly implementing
//! fully /etc/nsswitch.conf-respecting NSS functionality, we invoke the
//! `getent` command and parse its output.
//!
//! This file doesn't yet implement enumeration, but the `getent` command does,
//! so it's theoretically doable.

use libc::{c_char, c_int, c_void, gid_t, group, passwd, uid_t};
use rustix::path::DecInt;
use std::ffi::{CStr, OsStr};
use std::mem::align_of;
use std::os::unix::ffi::OsStrExt;
use std::process::Command;
use std::ptr::{copy_nonoverlapping, null_mut, write};
use std::str;

#[no_mangle]
unsafe extern "C" fn getpwnam_r(
    name: *const c_char,
    pwd: *mut passwd,
    buf: *mut c_char,
    buflen: usize,
    result: *mut *mut passwd,
) -> c_int {
    libc!(libc::getpwnam_r(name, pwd, buf, buflen, result));

    let name = OsStr::from_bytes(CStr::from_ptr(name).to_bytes());
    let mut command = Command::new("getent");
    command.arg("passwd").arg(name);
    getpw_r(command, pwd, buf, buflen, result)
}

#[no_mangle]
unsafe extern "C" fn getpwuid_r(
    uid: uid_t,
    pwd: *mut passwd,
    buf: *mut c_char,
    buflen: usize,
    result: *mut *mut passwd,
) -> c_int {
    libc!(libc::getpwuid_r(uid, pwd, buf, buflen, result));

    let dec_int = DecInt::new(uid);
    let name = OsStr::from_bytes(dec_int.as_bytes());
    let mut command = Command::new("getent");
    command.arg("passwd").arg(name);
    getpw_r(command, pwd, buf, buflen, result)
}

#[no_mangle]
unsafe extern "C" fn getgrnam_r(
    name: *const c_char,
    grp: *mut group,
    buf: *mut c_char,
    buflen: usize,
    result: *mut *mut group,
) -> c_int {
    libc!(libc::getgrnam_r(name, grp, buf, buflen, result));

    let name = OsStr::from_bytes(CStr::from_ptr(name).to_bytes());
    let mut command = Command::new("getent");
    command.arg("group").arg(name);
    getgr_r(command, grp, buf, buflen, result)
}

#[no_mangle]
unsafe extern "C" fn getgrgid_r(
    gid: gid_t,
    grp: *mut group,
    buf: *mut c_char,
    buflen: usize,
    result: *mut *mut group,
) -> c_int {
    libc!(libc::getgrgid_r(gid, grp, buf, buflen, result));

    let dec_int = DecInt::new(gid);
    let name = OsStr::from_bytes(dec_int.as_bytes());
    let mut command = Command::new("getent");
    command.arg("group").arg(name);
    getgr_r(command, grp, buf, buflen, result)
}

unsafe fn getpw_r(
    command: Command,
    pwd: *mut passwd,
    buf: *mut c_char,
    buflen: usize,
    result: *mut *mut passwd,
) -> c_int {
    let mut command = command;
    let output = match command.output() {
        Ok(output) => output,
        Err(_err) => {
            *result = null_mut();
            return libc::EIO;
        }
    };

    match output.status.code() {
        Some(0) => {}
        Some(2) => {
            // The username was not found.
            *result = null_mut();
            return 0;
        }
        Some(r) => panic!("unexpected exit status from `getent passwd`: {}", r),
        None => {
            *result = null_mut();
            return libc::EIO;
        }
    }

    let stdout = match str::from_utf8(&output.stdout) {
        Ok(stdout) => stdout,
        Err(_err) => return parse_error(result.cast()),
    };
    let stdout = match stdout.strip_suffix('\n') {
        Some(stdout) => stdout,
        None => return parse_error(result.cast()),
    };

    let mut parts = stdout.split(':');
    let mut buf = buf;
    let mut buflen = buflen;

    let part = match parts.next() {
        Some(part) => part,
        None => return parse_error(result.cast()),
    };
    if part.len() > buflen {
        return buffer_exhausted(result.cast());
    }
    let pw_name = buf;
    copy_nonoverlapping(part.as_ptr(), buf.cast(), part.len());
    buf = buf.add(part.len());
    write(buf, 0);
    buf = buf.add(1);
    buflen -= part.len() + 1;

    let part = match parts.next() {
        Some(part) => part,
        None => return parse_error(result.cast()),
    };
    if part.len() > buflen {
        return buffer_exhausted(result.cast());
    }
    let pw_passwd = buf;
    copy_nonoverlapping(part.as_ptr(), buf.cast(), part.len());
    buf = buf.add(part.len());
    write(buf, 0);
    buf = buf.add(1);
    buflen -= part.len() + 1;

    let part = match parts.next() {
        Some(part) => part,
        None => return parse_error(result.cast()),
    };
    let pw_uid = match part.parse() {
        Ok(pw_uid) => pw_uid,
        Err(_err) => return parse_error(result.cast()),
    };

    let part = match parts.next() {
        Some(part) => part,
        None => return parse_error(result.cast()),
    };
    let pw_gid = match part.parse() {
        Ok(pw_gid) => pw_gid,
        Err(_err) => return parse_error(result.cast()),
    };

    let part = match parts.next() {
        Some(part) => part,
        None => return parse_error(result.cast()),
    };
    if part.len() > buflen {
        return buffer_exhausted(result.cast());
    }
    let pw_gecos = buf;
    copy_nonoverlapping(part.as_ptr(), buf.cast(), part.len());
    buf = buf.add(part.len());
    write(buf, 0);
    buf = buf.add(1);
    buflen -= part.len() + 1;

    let part = match parts.next() {
        Some(part) => part,
        None => return parse_error(result.cast()),
    };
    if part.len() > buflen {
        return buffer_exhausted(result.cast());
    }
    let pw_dir = buf;
    copy_nonoverlapping(part.as_ptr(), buf.cast(), part.len());
    buf = buf.add(part.len());
    write(buf, 0);
    buf = buf.add(1);
    buflen -= part.len() + 1;

    let part = match parts.next() {
        Some(part) => part,
        None => return parse_error(result.cast()),
    };
    if part.len() > buflen {
        return buffer_exhausted(result.cast());
    }
    let pw_shell = buf;
    copy_nonoverlapping(part.as_ptr(), buf.cast(), part.len());
    buf = buf.add(part.len());
    write(buf, 0);

    write(
        pwd,
        passwd {
            pw_name,
            pw_passwd,
            pw_uid,
            pw_gid,
            pw_gecos,
            pw_dir,
            pw_shell,
        },
    );
    *result = pwd;
    0
}

unsafe fn getgr_r(
    command: Command,
    grp: *mut group,
    buf: *mut c_char,
    buflen: usize,
    result: *mut *mut group,
) -> c_int {
    let mut command = command;
    let output = match command.output() {
        Ok(output) => output,
        Err(_err) => {
            *result = null_mut();
            return libc::EIO;
        }
    };

    match output.status.code() {
        Some(0) => {}
        Some(2) => {
            // The username was not found.
            *result = null_mut();
            return 0;
        }
        Some(r) => panic!("unexpected exit status from `getent group`: {}", r),
        None => {
            *result = null_mut();
            return libc::EIO;
        }
    }

    let stdout = match str::from_utf8(&output.stdout) {
        Ok(stdout) => stdout,
        Err(_err) => return parse_error(result.cast()),
    };
    let stdout = match stdout.strip_suffix('\n') {
        Some(stdout) => stdout,
        None => return parse_error(result.cast()),
    };

    let mut parts = stdout.split(':');
    let mut buf = buf;
    let mut buflen = buflen;

    let part = match parts.next() {
        Some(part) => part,
        None => return parse_error(result.cast()),
    };
    if part.len() > buflen {
        return buffer_exhausted(result.cast());
    }
    let gr_name = buf;
    copy_nonoverlapping(part.as_ptr(), buf.cast(), part.len());
    buf = buf.add(part.len());
    write(buf, 0);
    buf = buf.add(1);
    buflen -= part.len() + 1;

    let part = match parts.next() {
        Some(part) => part,
        None => return parse_error(result.cast()),
    };
    if part.len() > buflen {
        return buffer_exhausted(result.cast());
    }
    let gr_passwd = buf;
    copy_nonoverlapping(part.as_ptr(), buf.cast(), part.len());
    buf = buf.add(part.len());
    write(buf, 0);
    buf = buf.add(1);
    buflen -= part.len() + 1;

    let part = match parts.next() {
        Some(part) => part,
        None => return parse_error(result.cast()),
    };
    let gr_gid = match part.parse() {
        Ok(pw_gid) => pw_gid,
        Err(_err) => return parse_error(result.cast()),
    };

    let part = match parts.next() {
        Some(part) => part,
        None => return parse_error(result.cast()),
    };
    if part.len() > buflen {
        return buffer_exhausted(result.cast());
    }

    let num_members = if part.is_empty() {
        0
    } else {
        part.split(',').count()
    };
    let pad = align_of::<*const c_char>() - (buf.addr()) % align_of::<*const c_char>();
    buf = buf.add(pad);
    buflen -= pad;
    let gr_mem = buf.cast::<*mut c_char>();
    buf = gr_mem.add(num_members + 1).cast::<c_char>();
    buflen -= buf.addr() - gr_mem.addr();

    let mut cur_mem = gr_mem;
    if num_members != 0 {
        for member in part.split(',') {
            *cur_mem = buf;
            cur_mem = cur_mem.add(1);
            copy_nonoverlapping(member.as_ptr(), buf.cast(), member.len());
            buf = buf.add(member.len());
            write(buf, 0);
            buf = buf.add(1);
            buflen -= member.len() + 1;
        }
    }
    write(cur_mem, null_mut());

    write(
        grp,
        group {
            gr_name,
            gr_passwd,
            gr_gid,
            gr_mem,
        },
    );
    *result = grp;
    0
}

#[cold]
unsafe fn buffer_exhausted(result: *mut *mut c_void) -> c_int {
    *result = null_mut();
    libc::ERANGE
}

#[cold]
unsafe fn parse_error(result: *mut *mut c_void) -> c_int {
    *result = null_mut();
    libc::EIO
}
