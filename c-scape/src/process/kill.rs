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

/// This function conforms to the [LSB `__libc_current_sigrtmax`] ABI.
///
/// [LSB `__libc_current_sigrtmax`]: https://refspecs.linuxbase.org/LSB_3.1.0/LSB-generic/LSB-generic/baselib---libc-current-sigrtmax-1.html
#[no_mangle]
unsafe extern "C" fn __libc_current_sigrtmax() -> c_int {
    libc!(libc::__libc_current_sigrtmax());

    // TODO: Upstream `NSIG` into the libc crate.
    #[cfg(not(any(target_arch = "mips", target_arch = "mips64")))]
    let nsig = 65;
    #[cfg(any(target_arch = "mips", target_arch = "mips64"))]
    let nsig = 128;

    nsig - 1
}
