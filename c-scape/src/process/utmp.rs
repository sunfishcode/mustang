use core::ptr::null_mut;

#[no_mangle]
unsafe extern "C" fn getutxent() -> *mut libc::utmpx {
    libc!(libc::getutxent());
    null_mut() // unimplemented
}

#[no_mangle]
unsafe extern "C" fn getutxid(_ut: *const libc::utmpx) -> *mut libc::utmpx {
    libc!(libc::getutxid(_ut));
    null_mut() // unimplemented
}

#[no_mangle]
unsafe extern "C" fn getutxline(_ut: *const libc::utmpx) -> *mut libc::utmpx {
    libc!(libc::getutxline(_ut));
    null_mut() // unimplemented
}

#[no_mangle]
unsafe extern "C" fn pututxline(_ut: *const libc::utmpx) -> *mut libc::utmpx {
    libc!(libc::pututxline(_ut));
    null_mut() // unimplemented
}

#[no_mangle]
unsafe extern "C" fn setutxent() {
    libc!(libc::setutxent());
    // unimplemented
}

#[no_mangle]
unsafe extern "C" fn endutxent() {
    libc!(libc::endutxent());
    // unimplemented
}

#[no_mangle]
unsafe extern "C" fn utmpxname(_file: *const libc::c_char) -> libc::c_int {
    libc!(libc::utmpxname(_file));
    0 // unimplemented
}
