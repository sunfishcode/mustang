use crate::convert_res;
use core::ptr::copy_nonoverlapping;
use core::slice;
use errno::{set_errno, Errno};
use libc::{c_char, c_double, c_int};

#[no_mangle]
unsafe extern "C" fn uname(buf: *mut libc::utsname) -> c_int {
    libc!(libc::uname(buf));

    let uname = rustix::system::uname();
    let buf = &mut *buf;

    let sysname = uname.sysname().to_bytes_with_nul();
    assert!(sysname.len() <= buf.sysname.len());
    copy_nonoverlapping(
        checked_cast!(sysname.as_ptr()),
        buf.sysname.as_mut_ptr(),
        sysname.len(),
    );

    let nodename = uname.nodename().to_bytes_with_nul();
    assert!(nodename.len() <= buf.nodename.len());
    copy_nonoverlapping(
        checked_cast!(nodename.as_ptr()),
        buf.nodename.as_mut_ptr(),
        nodename.len(),
    );

    let release = uname.release().to_bytes_with_nul();
    assert!(release.len() <= buf.release.len());
    copy_nonoverlapping(
        checked_cast!(release.as_ptr()),
        buf.release.as_mut_ptr(),
        release.len(),
    );

    let version = uname.version().to_bytes_with_nul();
    assert!(version.len() <= buf.version.len());
    copy_nonoverlapping(
        checked_cast!(version.as_ptr()),
        buf.version.as_mut_ptr(),
        version.len(),
    );

    let machine = uname.machine().to_bytes_with_nul();
    assert!(machine.len() <= buf.machine.len());
    copy_nonoverlapping(
        checked_cast!(machine.as_ptr()),
        buf.machine.as_mut_ptr(),
        machine.len(),
    );

    let domainname = uname.domainname().to_bytes_with_nul();
    assert!(domainname.len() <= buf.domainname.len());
    copy_nonoverlapping(
        checked_cast!(domainname.as_ptr()),
        buf.domainname.as_mut_ptr(),
        domainname.len(),
    );

    0
}

#[no_mangle]
unsafe extern "C" fn getloadavg(a: *mut c_double, n: c_int) -> c_int {
    libc!(libc::getloadavg(a, n));

    if n == 0 {
        return 0;
    }
    if n < 0 {
        return -1;
    }

    let info = rustix::system::sysinfo();
    let mut a = a;

    for i in 0..core::cmp::min(n as usize, info.loads.len()) {
        a.write((info.loads[i] as f64) / ((1 << libc::SI_LOAD_SHIFT) as f64));
        a = a.add(1);
    }

    n
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn gethostname(name: *mut c_char, len: usize) -> c_int {
    let uname = rustix::system::uname();
    let nodename = uname.nodename();
    if nodename.to_bytes().len() + 1 > len {
        set_errno(Errno(libc::ENAMETOOLONG));
        return -1;
    }
    libc::memcpy(
        name.cast(),
        nodename.to_bytes().as_ptr().cast(),
        nodename.to_bytes().len(),
    );
    *name.add(nodename.to_bytes().len()) = 0;
    0
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn sethostname(name: *mut c_char, len: usize) -> c_int {
    let slice = slice::from_raw_parts(name.cast(), len);
    match convert_res(rustix::system::sethostname(slice)) {
        Some(()) => 0,
        None => -1,
    }
}
