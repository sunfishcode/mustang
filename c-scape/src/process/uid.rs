use libc::{c_int, uid_t};

#[no_mangle]
unsafe extern "C" fn getuid() -> uid_t {
    libc!(libc::getuid());
    rustix::process::getuid().as_raw()
}

#[no_mangle]
unsafe extern "C" fn setuid(uid: uid_t) -> c_int {
    libc!(libc::setuid(uid));

    // rustix has a `setuid` function, but it just wraps the Linux syscall
    // which sets a per-thread UID rather than the whole process UID. Linux
    // expects libc's to have logic to set the UID for all the threads.
    unimplemented!("setuid")
}
