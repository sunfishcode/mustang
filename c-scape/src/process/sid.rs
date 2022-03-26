use libc::pid_t;

use crate::convert_res;

#[no_mangle]
unsafe extern "C" fn getsid(pid: pid_t) -> pid_t {
    libc!(libc::getsid(pid));
    unimplemented!("getsid")
}

#[no_mangle]
unsafe extern "C" fn setsid() -> pid_t {
    libc!(libc::setsid());

    match convert_res(rustix::process::setsid()) {
        Some(v) => v.as_raw_nonzero().get() as _,
        None => -1,
    }
}