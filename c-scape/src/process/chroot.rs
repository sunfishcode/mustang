use core::ffi::CStr;

use libc::{c_char, c_int};

use crate::convert_res;

#[no_mangle]
unsafe extern "C" fn chroot(path: *const c_char) -> c_int {
    libc!(libc::chroot(path));

    let path = CStr::from_ptr(path.cast());
    match convert_res(rustix::process::chroot(path)) {
        Some(()) => 0,
        None => -1,
    }
}
