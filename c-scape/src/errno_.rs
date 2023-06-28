use alloc::borrow::ToOwned;
use alloc::format;
use core::cell::SyncUnsafeCell;
use core::ptr::copy_nonoverlapping;
use libc::{c_char, c_int};

/// Return the address of the thread-local `errno` state.
///
/// This function conforms to the [LSB `__errno_location`] ABI.
///
/// [LSB `__errno_location`]: https://refspecs.linuxbase.org/LSB_3.1.0/LSB-generic/LSB-generic/baselib-errno-location-1.html
#[no_mangle]
unsafe extern "C" fn __errno_location() -> *mut c_int {
    libc!(libc::__errno_location());

    #[cfg_attr(feature = "threads", thread_local)]
    static mut ERRNO: i32 = 0;
    &mut ERRNO
}

#[no_mangle]
unsafe extern "C" fn strerror(errnum: c_int) -> *mut c_char {
    libc!(libc::strerror(errnum));

    static STORAGE: SyncUnsafeCell<[c_char; 256]> = SyncUnsafeCell::new([0; 256]);

    let storage = SyncUnsafeCell::get(&STORAGE);
    __xpg_strerror_r(errnum, (*storage).as_mut_ptr(), (*storage).len());
    (*storage).as_mut_ptr()
}

// <https://refspecs.linuxfoundation.org/LSB_3.2.0/LSB-Core-generic/LSB-Core-generic/baselib-xpg-strerror_r-3.html>
#[no_mangle]
unsafe extern "C" fn __xpg_strerror_r(errnum: c_int, buf: *mut c_char, buflen: usize) -> c_int {
    libc!(libc::strerror_r(errnum, buf, buflen));

    if buflen == 0 {
        return libc::ERANGE;
    }

    let message = match crate::error_str::error_str(rustix::io::Errno::from_raw_os_error(errnum)) {
        Some(s) => s.to_owned(),
        None => format!("Unknown error {}", errnum),
    };

    let min = core::cmp::min(buflen - 1, message.len());
    copy_nonoverlapping(message.as_ptr().cast(), buf, min);
    buf.add(min).write(b'\0' as libc::c_char);
    0
}
