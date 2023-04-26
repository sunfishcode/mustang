use libc::{c_int, gid_t};

#[no_mangle]
unsafe extern "C" fn getgid() -> gid_t {
    libc!(libc::getgid());
    rustix::process::getgid().as_raw()
}

#[no_mangle]
unsafe extern "C" fn setgid(_gid: gid_t) -> c_int {
    libc!(libc::setgid(_gid));
    unimplemented!("setgid")
}
