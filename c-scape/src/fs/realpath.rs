use core::ffi::CStr;

use core::ptr::null_mut;
use errno::{set_errno, Errno};
use libc::{c_char, c_void, malloc, memcpy};

#[no_mangle]
unsafe extern "C" fn realpath(path: *const c_char, resolved_path: *mut c_char) -> *mut c_char {
    libc!(libc::realpath(path, resolved_path));

    let mut buf = [0; libc::PATH_MAX as usize];
    match realpath_ext::realpath_raw(
        CStr::from_ptr(path.cast()).to_bytes(),
        &mut buf,
        realpath_ext::RealpathFlags::empty(),
    ) {
        Ok(len) => {
            if resolved_path.is_null() {
                let ptr = malloc(len + 1).cast::<u8>();
                if ptr.is_null() {
                    set_errno(Errno(libc::ENOMEM));
                    return null_mut();
                }
                core::slice::from_raw_parts_mut(ptr, len).copy_from_slice(&buf[..len]);
                *ptr.add(len) = b'\0';
                ptr.cast::<c_char>()
            } else {
                memcpy(
                    resolved_path.cast::<c_void>(),
                    buf[..len].as_ptr().cast::<c_void>(),
                    len,
                );
                *resolved_path.add(len) = b'\0' as _;
                resolved_path
            }
        }
        Err(err) => {
            set_errno(Errno(err));
            null_mut()
        }
    }
}
