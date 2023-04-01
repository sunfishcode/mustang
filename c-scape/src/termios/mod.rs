//! Termios APIs
//!
//! TODO: Clean this up. I think I made the wrong call in rustix, and `Termios`
//! should contain the `termios2` fields. When that's cleaned up, this file can
//! be significantly cleaned up.

use crate::{convert_res, set_errno, Errno};
use libc::c_int;
use rustix::fd::BorrowedFd;
use rustix::termios::{OptionalActions, Termios, Termios2};

#[no_mangle]
unsafe extern "C" fn cfmakeraw(termios_p: *mut libc::termios) {
    libc!(libc::cfmakeraw(termios_p));

    let old_termios = *termios_p;
    let mut termios = Termios {
        c_cc: [
            old_termios.c_cc[0],
            old_termios.c_cc[1],
            old_termios.c_cc[2],
            old_termios.c_cc[3],
            old_termios.c_cc[4],
            old_termios.c_cc[5],
            old_termios.c_cc[6],
            old_termios.c_cc[7],
            old_termios.c_cc[8],
            old_termios.c_cc[9],
            old_termios.c_cc[10],
            old_termios.c_cc[11],
            old_termios.c_cc[12],
            old_termios.c_cc[13],
            old_termios.c_cc[14],
            old_termios.c_cc[15],
            old_termios.c_cc[16],
            old_termios.c_cc[17],
            old_termios.c_cc[18],
        ],
        c_cflag: old_termios.c_cflag,
        c_iflag: old_termios.c_iflag,
        c_oflag: old_termios.c_oflag,
        c_lflag: old_termios.c_lflag,
        c_line: old_termios.c_line,
    };

    rustix::termios::cfmakeraw(&mut termios);

    (*termios_p) = libc::termios {
        c_cc: [
            termios.c_cc[0],
            termios.c_cc[1],
            termios.c_cc[2],
            termios.c_cc[3],
            termios.c_cc[4],
            termios.c_cc[5],
            termios.c_cc[6],
            termios.c_cc[7],
            termios.c_cc[8],
            termios.c_cc[9],
            termios.c_cc[10],
            termios.c_cc[11],
            termios.c_cc[12],
            termios.c_cc[13],
            termios.c_cc[14],
            termios.c_cc[15],
            termios.c_cc[16],
            termios.c_cc[17],
            termios.c_cc[18],
            old_termios.c_cc[19],
            old_termios.c_cc[20],
            old_termios.c_cc[21],
            old_termios.c_cc[22],
            old_termios.c_cc[23],
            old_termios.c_cc[24],
            old_termios.c_cc[25],
            old_termios.c_cc[26],
            old_termios.c_cc[27],
            old_termios.c_cc[28],
            old_termios.c_cc[29],
            old_termios.c_cc[30],
            old_termios.c_cc[31],
        ],
        c_cflag: termios.c_cflag,
        c_iflag: termios.c_iflag,
        c_oflag: termios.c_oflag,
        c_lflag: termios.c_lflag,
        c_line: termios.c_line,
        c_ispeed: old_termios.c_ispeed,
        c_ospeed: old_termios.c_ospeed,
    };
}

#[no_mangle]
unsafe extern "C" fn tcgetattr(fd: c_int, termios_p: *mut libc::termios) -> c_int {
    libc!(libc::tcgetattr(fd, termios_p));

    match convert_res(rustix::termios::tcgetattr2(BorrowedFd::borrow_raw(fd))) {
        Some(termios) => {
            *termios_p = libc::termios {
                c_cc: [
                    termios.c_cc[0],
                    termios.c_cc[1],
                    termios.c_cc[2],
                    termios.c_cc[3],
                    termios.c_cc[4],
                    termios.c_cc[5],
                    termios.c_cc[6],
                    termios.c_cc[7],
                    termios.c_cc[8],
                    termios.c_cc[9],
                    termios.c_cc[10],
                    termios.c_cc[11],
                    termios.c_cc[12],
                    termios.c_cc[13],
                    termios.c_cc[14],
                    termios.c_cc[15],
                    termios.c_cc[16],
                    termios.c_cc[17],
                    termios.c_cc[18],
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ],
                c_cflag: termios.c_cflag,
                c_iflag: termios.c_iflag,
                c_oflag: termios.c_oflag,
                c_lflag: termios.c_lflag,
                c_line: termios.c_line,
                c_ispeed: termios.c_ispeed,
                c_ospeed: termios.c_ospeed,
            };
            0
        }
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn tcsetattr(
    fd: c_int,
    optional_actions: c_int,
    termios_p: *const libc::termios,
) -> c_int {
    libc!(libc::tcsetattr(fd, optional_actions, termios_p));

    let optional_actions = match optional_actions {
        libc::TCSANOW => OptionalActions::Now,
        libc::TCSADRAIN => OptionalActions::Drain,
        libc::TCSAFLUSH => OptionalActions::Flush,
        _ => {
            set_errno(Errno(libc::EINVAL));
            return -1;
        }
    };

    let termios = *termios_p;
    let termios = Termios2 {
        c_cc: [
            termios.c_cc[0],
            termios.c_cc[1],
            termios.c_cc[2],
            termios.c_cc[3],
            termios.c_cc[4],
            termios.c_cc[5],
            termios.c_cc[6],
            termios.c_cc[7],
            termios.c_cc[8],
            termios.c_cc[9],
            termios.c_cc[10],
            termios.c_cc[11],
            termios.c_cc[12],
            termios.c_cc[13],
            termios.c_cc[14],
            termios.c_cc[15],
            termios.c_cc[16],
            termios.c_cc[17],
            termios.c_cc[18],
        ],
        c_cflag: termios.c_cflag,
        c_iflag: termios.c_iflag,
        c_oflag: termios.c_oflag,
        c_lflag: termios.c_lflag,
        c_line: termios.c_line,
        c_ispeed: termios.c_ispeed,
        c_ospeed: termios.c_ospeed,
    };

    match convert_res(rustix::termios::tcsetattr2(
        BorrowedFd::borrow_raw(fd),
        optional_actions,
        &termios,
    )) {
        Some(()) => 0,
        None => -1,
    }
}
