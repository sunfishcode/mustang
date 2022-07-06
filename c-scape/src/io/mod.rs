#[cfg(not(target_os = "wasi"))]
mod dup;
// #[cfg(any(target_os = "android", target_os = "linux"))]
// mod epoll;
mod isatty;
mod pipe;
mod poll;
mod read;
mod write;

use rustix::fd::{BorrowedFd, IntoRawFd};
use rustix::io::EventfdFlags;

use core::convert::TryInto;
use libc::{c_int, c_long, c_uint, loff_t};

use crate::convert_res;

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn ioctl(fd: c_int, request: c_long, mut args: ...) -> c_int {
    const TCGETS: c_long = libc::TCGETS as c_long;
    const FIONBIO: c_long = libc::FIONBIO as c_long;
    const TIOCGWINSZ: c_long = libc::TIOCGWINSZ as c_long;
    match request {
        TCGETS => {
            libc!(libc::ioctl(fd, libc::TCGETS));
            let fd = BorrowedFd::borrow_raw(fd);
            match convert_res(rustix::termios::tcgetattr(&fd)) {
                Some(x) => {
                    args.arg::<*mut rustix::termios::Termios>().write(x);
                    0
                }
                None => -1,
            }
        }
        FIONBIO => {
            libc!(libc::ioctl(fd, libc::FIONBIO));
            let fd = BorrowedFd::borrow_raw(fd);
            let ptr = args.arg::<*mut c_int>();
            let value = *ptr != 0;
            match convert_res(rustix::io::ioctl_fionbio(&fd, value)) {
                Some(()) => 0,
                None => -1,
            }
        }
        TIOCGWINSZ => {
            libc!(libc::ioctl(fd, libc::TIOCGWINSZ));
            let fd = BorrowedFd::borrow_raw(fd);
            match convert_res(rustix::termios::tcgetwinsize(&fd)) {
                Some(x) => {
                    args.arg::<*mut rustix::termios::Winsize>().write(x);
                    0
                }
                None => -1,
            }
        }
        _ => panic!("unrecognized ioctl({})", request),
    }
}

#[cfg(any(target_os = "android", target_os = "linux"))]
#[no_mangle]
unsafe extern "C" fn sendfile() {
    //libc!(libc::sendfile());
    unimplemented!("sendfile")
}

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
    unimplemented!("splice");
}

#[cfg(feature = "net")]
#[no_mangle]
unsafe extern "C" fn sendmsg() {
    //libc!(libc::sendmsg());
    unimplemented!("sendmsg")
}

#[cfg(feature = "net")]
#[no_mangle]
unsafe extern "C" fn recvmsg() {
    //libc!(libc::recvmsg());
    unimplemented!("recvmsg")
}

#[cfg(any(target_os = "android", target_os = "linux"))]
#[no_mangle]
unsafe extern "C" fn eventfd(initval: c_uint, flags: c_int) -> c_int {
    libc!(libc::eventfd(initval, flags));
    let flags = EventfdFlags::from_bits(flags.try_into().unwrap()).unwrap();
    match convert_res(rustix::io::eventfd(initval, flags)) {
        Some(fd) => fd.into_raw_fd(),
        None => -1,
    }
}
