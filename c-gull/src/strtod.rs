use libc::{c_char, c_double, c_float};

#[no_mangle]
unsafe extern "C" fn strtof(nptr: *const c_char, endptr: *mut *mut c_char) -> c_float {
    libc!(libc::strtof(nptr, endptr));

    unimplemented!("strtof")
}

#[no_mangle]
unsafe extern "C" fn strtod(nptr: *const c_char, endptr: *mut *mut c_char) -> c_double {
    libc!(libc::strtod(nptr, endptr));

    unimplemented!("strtod")
}
