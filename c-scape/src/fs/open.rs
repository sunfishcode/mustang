use core::ffi::CStr;
use rustix::fd::{BorrowedFd, IntoRawFd};
use rustix::fs::{cwd, Mode, OFlags};

use libc::{c_char, c_int};

use crate::convert_res;

// This has to be a macro because we can't delegate C varadic functions.
// This allows us to reduce code duplication as all of the `open` variants
// should morally delegate to openat64.
macro_rules! openat_impl {
    ($fd:expr, $pathname:ident, $flags:ident, $args:ident) => {{
        let flags = OFlags::from_bits($flags as _).unwrap();
        let mode = if flags.contains(OFlags::CREATE) {
            let mode: libc::mode_t = $args.arg();
            Mode::from_bits((mode & !libc::S_IFMT) as _).unwrap()
        } else {
            Mode::empty()
        };
        match convert_res(rustix::fs::openat(
            &$fd,
            CStr::from_ptr($pathname.cast()),
            flags,
            mode,
        )) {
            Some(fd) => fd.into_raw_fd(),
            None => -1,
        }
    }};
}

// we open all files with O_LARGEFILE as that is what Rustix does
// hopefully this doesn't break any C programs
#[no_mangle]
unsafe extern "C" fn open(pathname: *const c_char, flags: c_int, mut args: ...) -> c_int {
    libc!(libc::open(pathname, flags, args));

    openat_impl!(cwd(), pathname, flags, args)
}

#[no_mangle]
unsafe extern "C" fn open64(pathname: *const c_char, flags: c_int, mut args: ...) -> c_int {
    libc!(libc::open64(pathname, flags, args));

    openat_impl!(cwd(), pathname, flags, args)
}

// same behavior with `O_LARGEFILE` as open/open64
#[no_mangle]
unsafe extern "C" fn openat(
    fd: c_int,
    pathname: *const c_char,
    flags: c_int,
    mut args: ...
) -> c_int {
    libc!(libc::openat(fd, pathname, flags, args));

    openat_impl!(BorrowedFd::borrow_raw(fd), pathname, flags, args)
}

#[no_mangle]
unsafe extern "C" fn openat64(
    fd: c_int,
    pathname: *const c_char,
    flags: c_int,
    mut args: ...
) -> c_int {
    libc!(libc::openat64(fd, pathname, flags, args));

    openat_impl!(BorrowedFd::borrow_raw(fd), pathname, flags, args)
}

#[no_mangle]
unsafe extern "C" fn close(fd: c_int) -> c_int {
    libc!(libc::close(fd));

    rustix::io::close(fd);
    0
}
