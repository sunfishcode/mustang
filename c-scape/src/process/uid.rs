use libc::uid_t;
use rustix::process::Uid;

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
