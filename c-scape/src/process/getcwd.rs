use alloc::vec::Vec;
use core::ptr::null_mut;
use errno::{set_errno, Errno};
use libc::{c_char, memcpy};

use crate::convert_res;

#[no_mangle]
unsafe extern "C" fn getcwd(buf: *mut c_char, len: usize) -> *mut c_char {
    libc!(libc::getcwd(buf, len));

    match convert_res(rustix::process::getcwd(Vec::new())) {
        Some(path) => {
            let path = path.as_bytes();
            if path.len() + 1 <= len {
                memcpy(buf.cast(), path.as_ptr().cast(), path.len());
                *buf.add(path.len()) = 0;
                buf
            } else {
                set_errno(Errno(libc::ERANGE));
                null_mut()
            }
        }
        None => null_mut(),
    }
}
