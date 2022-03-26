#[no_mangle]
unsafe extern "C" fn getgroups() {
    //libc!(libc::getgroups());
    unimplemented!("getgroups")
}

#[cfg(any(target_os = "android", target_os = "linux"))]
#[no_mangle]
unsafe extern "C" fn setgroups() {
    //libc!(libc::setgroups());
    unimplemented!("setgroups")
}