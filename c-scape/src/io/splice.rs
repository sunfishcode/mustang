use rustix::fd::BorrowedFd;
use rustix::pipe::SpliceFlags;

use libc::{c_int, c_uint, loff_t};

use crate::convert_res;

#[cfg(any(target_os = "android", target_os = "linux"))]
#[no_mangle]
unsafe extern "C" fn splice(
    fd_in: c_int,
    off_in: *mut loff_t,
    fd_out: c_int,
    off_out: *mut loff_t,
    len: usize,
    flags: c_uint,
) -> isize {
    libc!(libc::splice(fd_in, off_in, fd_out, off_out, len, flags));

    let off_in: *mut u64 = checked_cast!(off_in);
    let off_out: *mut u64 = checked_cast!(off_out);

    match convert_res(rustix::pipe::splice(
        BorrowedFd::borrow_raw(fd_in),
        off_in.as_mut(),
        BorrowedFd::borrow_raw(fd_out),
        off_out.as_mut(),
        len,
        SpliceFlags::from_bits(flags as _).unwrap(),
    )) {
        Some(num) => num.try_into().unwrap(),
        None => -1,
    }
}

#[cfg(any(target_os = "android", target_os = "linux"))]
#[no_mangle]
unsafe extern "C" fn vmsplice(
    fd: c_int,
    iov: *const libc::iovec,
    nr_segs: usize,
    flags: libc::c_uint,
) -> libc::ssize_t {
    libc!(libc::vmsplice(fd, iov, nr_segs, flags));

    let bufs = core::slice::from_raw_parts(iov.cast::<rustix::pipe::IoSliceRaw>(), nr_segs);
    let flags = SpliceFlags::from_bits(flags).unwrap();
    match convert_res(rustix::pipe::vmsplice(
        BorrowedFd::borrow_raw(fd),
        bufs,
        flags,
    )) {
        Some(num) => num.try_into().unwrap(),
        None => -1,
    }
}
