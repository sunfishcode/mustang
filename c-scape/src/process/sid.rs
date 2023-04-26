use crate::convert_res;
use libc::pid_t;
use rustix::process::Pid;

#[no_mangle]
unsafe extern "C" fn getsid(pid: pid_t) -> pid_t {
    libc!(libc::getsid(pid));

    match convert_res(rustix::process::getsid(Pid::from_raw(pid as _))) {
        Some(v) => v.as_raw_nonzero().get() as _,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn setsid() -> pid_t {
    libc!(libc::setsid());

    match convert_res(rustix::process::setsid()) {
        Some(v) => v.as_raw_nonzero().get() as _,
        None => -1,
    }
}
