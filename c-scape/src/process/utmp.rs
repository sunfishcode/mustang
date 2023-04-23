use core::ptr::null_mut;
use errno::{set_errno, Errno};

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

    set_errno(Errno(libc::ENOTSUP));
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
