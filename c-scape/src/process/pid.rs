use crate::convert_res;
use libc::{c_int, pid_t};
use rustix::process::Pid;

#[no_mangle]
unsafe extern "C" fn getpid() -> pid_t {
    libc!(libc::getpid());
    rustix::process::getpid().as_raw_nonzero().get() as _
}

#[no_mangle]
unsafe extern "C" fn setpid() {
    unimplemented!("setpid");
}

#[no_mangle]
unsafe extern "C" fn getppid() -> pid_t {
    libc!(libc::getppid());
    Pid::as_raw(rustix::process::getppid()) as _
}

#[no_mangle]
unsafe extern "C" fn setpgid(pid: pid_t, pgid: pid_t) -> c_int {
    libc!(libc::setpgid(pid, pgid));
    unimplemented!("setpgid")
}

#[no_mangle]
unsafe extern "C" fn getpgid(pid: pid_t) -> pid_t {
    libc!(libc::getpgid(pid));

    match convert_res(rustix::process::getpgid(Pid::from_raw(pid as _))) {
        Some(pid) => Pid::as_raw(Some(pid)) as _,
        None => -1,
    }
}
