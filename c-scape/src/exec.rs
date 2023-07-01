use crate::convert_res;
use libc::{c_int, c_void};

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn fork() -> c_int {
    libc!(libc::fork());
    match convert_res(crate::at_fork::fork()) {
        Some(Some(pid)) => pid.as_raw_nonzero().get() as c_int,
        Some(None) => 0,
        None => -1,
    }
}

// <https://refspecs.linuxbase.org/LSB_5.0.0/LSB-Core-generic/LSB-Core-generic/baselib---register-atfork.html>
#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn __register_atfork(
    prepare: Option<unsafe extern "C" fn()>,
    parent: Option<unsafe extern "C" fn()>,
    child: Option<unsafe extern "C" fn()>,
    _dso_handle: *mut c_void,
) -> c_int {
    //libc!(libc::__register_atfork(prepare, parent, child, _dso_handle));
    crate::at_fork::at_fork(prepare, parent, child);
    0
}

#[no_mangle]
unsafe extern "C" fn clone3() {
    //libc!(libc::clone3());
    unimplemented!("clone3")
}
