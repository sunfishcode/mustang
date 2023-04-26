use libc::{c_int, gid_t};

#[no_mangle]
unsafe extern "C" fn getegid() -> gid_t {
    libc!(libc::getegid());
    rustix::process::getegid().as_raw()
}

#[no_mangle]
unsafe extern "C" fn setegid(_gid: gid_t) -> c_int {
    libc!(libc::setegid(_gid));
    unimplemented!("setegid")
}
