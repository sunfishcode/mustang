use crate::convert_res;
use errno::{set_errno, Errno};
use libc::{c_int, c_uint, id_t};
use rustix::process::{Pid, Uid};

#[no_mangle]
unsafe extern "C" fn getpriority(which: c_uint, who: id_t) -> c_int {
    libc!(libc::getpriority(which, who));

    let result = match which {
        libc::PRIO_PROCESS => rustix::process::getpriority_process(Pid::from_raw(who)),
        libc::PRIO_PGRP => rustix::process::getpriority_pgrp(Pid::from_raw(who)),
        libc::PRIO_USER => rustix::process::getpriority_user(Uid::from_raw(who)),
        _ => {
            set_errno(Errno(libc::EINVAL));
            return -1;
        }
    };

    match convert_res(result) {
        Some(prio) => prio,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn setpriority(which: c_uint, who: id_t, prio: c_int) -> c_int {
    libc!(libc::setpriority(which, who, prio));

    let result = match which {
        libc::PRIO_PROCESS => rustix::process::setpriority_process(Pid::from_raw(who), prio),
        libc::PRIO_PGRP => rustix::process::setpriority_pgrp(Pid::from_raw(who), prio),
        libc::PRIO_USER => rustix::process::setpriority_user(Uid::from_raw(who), prio),
        _ => {
            set_errno(Errno(libc::EINVAL));
            return -1;
        }
    };

    match convert_res(result) {
        Some(()) => 0,
        None => -1,
    }
}
