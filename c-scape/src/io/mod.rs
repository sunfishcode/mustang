#[cfg(not(target_os = "wasi"))]
mod dup;
#[cfg(any(target_os = "android", target_os = "linux"))]
mod epoll;
mod isatty;
mod pipe;
mod poll;
mod read;
mod splice;
#[cfg(any(target_os = "android", target_os = "linux"))]
mod timerfd;
mod write;

use rustix::fd::{BorrowedFd, IntoRawFd};
use rustix::io::EventfdFlags;

use core::convert::TryInto;
use libc::{c_int, c_long, c_uint};

use crate::convert_res;

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn ioctl(fd: c_int, request: c_long, mut args: ...) -> c_int {
    const TCGETS: c_long = libc::TCGETS as c_long;
    const FIONBIO: c_long = libc::FIONBIO as c_long;
    const TIOCGWINSZ: c_long = libc::TIOCGWINSZ as c_long;
    #[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
    const FICLONE: c_long = libc::FICLONE as c_long;
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
            let ptr = args.arg::<*mut c_int>();
            let value = *ptr != 0;
            libc!(libc::ioctl(fd, libc::FIONBIO, value as c_int));
            let fd = BorrowedFd::borrow_raw(fd);
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
        // TODO: Enable this on more architectures when libc is updated.
        #[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
        FICLONE => {
            let src_fd = args.arg::<c_int>();
            libc!(libc::ioctl(fd, libc::FICLONE, src_fd));
            let fd = BorrowedFd::borrow_raw(fd);
            let src_fd = BorrowedFd::borrow_raw(src_fd);
            match convert_res(rustix::io::ioctl_ficlone(&fd, &src_fd)) {
                Some(()) => 0,
                None => -1,
            }
        }
        _ => panic!("unrecognized ioctl({})", request),
    }
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
