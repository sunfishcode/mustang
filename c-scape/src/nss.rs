#[cfg(not(feature = "std"))] // Avoid conflicting with c-gull's more complete `getpwuid_r`.
#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn getpwuid_r(
    _uid: uid_t,
    _pwd: *mut passwd,
    _buf: *mut c_char,
    _buflen: usize,
    _result: *mut *mut passwd,
) -> c_int {
    libc!(libc::getpwuid_r(_uid, _pwd, _buf, _buflen, _result));

    // `getpwuid_r` is currently implemented in c-gull.
    unimplemented!("getpwuid_r without the \"std\" feature")
}
