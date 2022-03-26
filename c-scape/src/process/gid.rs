use libc::gid_t;

#[no_mangle]
unsafe extern "C" fn getgid() -> gid_t {
    libc!(libc::getgid());
    rustix::process::getgid().as_raw()
}

#[no_mangle]
unsafe extern "C" fn setgid() {
    //libc!(libc::setgid());
    unimplemented!("setgid")
}