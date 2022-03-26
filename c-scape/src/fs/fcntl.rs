use rustix::fd::{BorrowedFd, IntoRawFd};
use rustix::fs::{FdFlags, OFlags};

use libc::{c_int};

use crate::convert_res;

#[no_mangle]
unsafe extern "C" fn fcntl(fd: c_int, cmd: c_int, mut args: ...) -> c_int {
    match cmd {
        libc::F_GETFL => {
            libc!(libc::fcntl(fd, libc::F_GETFL));
            let fd = BorrowedFd::borrow_raw(fd);
            match convert_res(rustix::fs::fcntl_getfl(&fd)) {
                Some(flags) => flags.bits() as _,
                None => -1,
            }
        }
        libc::F_SETFL => {
            let flags = args.arg::<c_int>();
            libc!(libc::fcntl(fd, libc::F_SETFL, flags));
            let fd = BorrowedFd::borrow_raw(fd);
            match convert_res(rustix::fs::fcntl_setfl(
                &fd,
                OFlags::from_bits(flags as _).unwrap(),
            )) {
                Some(()) => 0,
                None => -1,
            }
        }
        libc::F_GETFD => {
            libc!(libc::fcntl(fd, libc::F_GETFD));
            let fd = BorrowedFd::borrow_raw(fd);
            match convert_res(rustix::fs::fcntl_getfd(&fd)) {
                Some(flags) => flags.bits() as _,
                None => -1,
            }
        }
        libc::F_SETFD => {
            let flags = args.arg::<c_int>();
            libc!(libc::fcntl(fd, libc::F_SETFD, flags));
            let fd = BorrowedFd::borrow_raw(fd);
            match convert_res(rustix::fs::fcntl_setfd(
                &fd,
                FdFlags::from_bits(flags as _).unwrap(),
            )) {
                Some(()) => 0,
                None => -1,
            }
        }
        #[cfg(not(target_os = "wasi"))]
        libc::F_DUPFD_CLOEXEC => {
            let arg = args.arg::<c_int>();
            libc!(libc::fcntl(fd, libc::F_DUPFD_CLOEXEC, arg));
            let fd = BorrowedFd::borrow_raw(fd);
            match convert_res(rustix::fs::fcntl_dupfd_cloexec(&fd, arg)) {
                Some(fd) => fd.into_raw_fd(),
                None => -1,
            }
        }
        _ => panic!("unrecognized fnctl({})", cmd),
    }
}