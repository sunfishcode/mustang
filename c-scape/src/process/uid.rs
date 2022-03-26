use libc::uid_t;

#[no_mangle]
unsafe extern "C" fn getuid() -> uid_t {
    libc!(libc::getuid());
    rustix::process::getuid().as_raw()
}

#[no_mangle]
unsafe extern "C" fn setuid() {
    //libc!(libc::setuid());
    unimplemented!("setuid")
}
