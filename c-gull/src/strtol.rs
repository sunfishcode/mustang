use libc::{c_char, c_int, c_long, c_longlong, c_ulong, c_ulonglong};

#[no_mangle]
unsafe extern "C" fn strtol(nptr: *const c_char, endptr: *mut *mut c_char, base: c_int) -> c_long {
    libc!(libc::strtol(nptr, endptr, base));

    unimplemented!("strtol")
}

#[no_mangle]
unsafe extern "C" fn strtoll(
    _nptr: *const c_char,
    _endptr: *mut *mut c_char,
    _base: c_int,
) -> c_longlong {
    //libc!(libc::strtoll(nptr, endptr, base));

    unimplemented!("strtoll")
}

#[no_mangle]
unsafe extern "C" fn strtoul(
    nptr: *const c_char,
    endptr: *mut *mut c_char,
    base: c_int,
) -> c_ulong {
    libc!(libc::strtoul(nptr, endptr, base));

    unimplemented!("strtoul")
}

#[no_mangle]
unsafe extern "C" fn strtoull(
    _nptr: *const c_char,
    _endptr: *mut *mut c_char,
    _base: c_int,
) -> c_ulonglong {
    //libc!(libc::strtoull(nptr, endptr, base));

    unimplemented!("strtoull")
}
