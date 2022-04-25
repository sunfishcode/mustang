use rustix::process::{Pid, Signal};

use errno::{set_errno, Errno};
use libc::{c_int, pid_t};

use crate::convert_res;

#[no_mangle]
unsafe extern "C" fn kill(pid: pid_t, sig: c_int) -> c_int {
    libc!(libc::kill(pid, sig));

    let sig = if let Some(sig) = Signal::from_raw(sig) {
        sig
    } else {
        set_errno(Errno(libc::EINVAL));
        return -1;
    };

    let res = match pid {
        n if n > 0 => {
            rustix::process::kill_process(Pid::from_raw(pid.unsigned_abs()).unwrap(), sig)
        }
        n if n < 0 => {
            rustix::process::kill_process_group(Pid::from_raw(pid.unsigned_abs()).unwrap(), sig)
        }
        _ => rustix::process::kill_current_process_group(sig),
    };

    match convert_res(res) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn killpg(pgid: pid_t, sig: c_int) -> c_int {
    libc!(libc::killpg(pgid, sig));

    if pgid < 0 {
        set_errno(Errno(libc::EINVAL));
        return -1;
    }

    kill(-pgid, sig)
}
