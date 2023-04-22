//! The `printf` family of functions.
//!
//! This code is highly experimental.
//!
//! This uses the `printf_compat` crate, which [has differences with glibc].
//!
//! The `fprintf` implementation doesn't have a real `FILE` type yet.
//!
//! Caution is indicated.
//!
//! [has differences with glibc]: https://docs.rs/printf-compat/0.1.1/printf_compat/output/fn.fmt_write.html#differences

use libc::{c_char, c_int, c_void};
use printf_compat::{format, output};
use rustix::fd::{FromRawFd, IntoRawFd};
use std::cmp::min;
use std::ffi::{CStr, VaList};
use std::io::{stderr as rust_stderr, stdout as rust_stdout, Write};
use std::ptr::copy_nonoverlapping;

#[no_mangle]
unsafe extern "C" fn puts(s: *const c_char) -> c_int {
    libc!(libc::puts(s));

    match rust_stdout().write_all(CStr::from_ptr(s).to_bytes()) {
        Ok(()) => 1,
        Err(_err) => libc::EOF,
    }
}

#[no_mangle]
unsafe extern "C" fn printf(fmt: *const c_char, mut args: ...) -> c_int {
    let va_list = args.as_va_list();
    vprintf(fmt, va_list)
}

#[no_mangle]
unsafe extern "C" fn vprintf(fmt: *const c_char, va_list: VaList) -> c_int {
    //libc!(libc::vprintf(fmt, va_list));

    format(fmt, va_list, output::io_write(&mut rust_stdout()))
}

#[no_mangle]
unsafe extern "C" fn sprintf(ptr: *mut c_char, fmt: *const c_char, mut args: ...) -> c_int {
    let va_list = args.as_va_list();
    vsprintf(ptr, fmt, va_list)
}

#[no_mangle]
unsafe extern "C" fn vsprintf(ptr: *mut c_char, fmt: *const c_char, va_list: VaList) -> c_int {
    //libc!(libc::vsprintf(ptr, fmt, va_list));

    let mut out = String::new();
    let num_bytes = format(fmt, va_list, output::fmt_write(&mut out));
    copy_nonoverlapping(out.as_ptr(), ptr.cast(), num_bytes as usize);
    num_bytes
}

#[no_mangle]
unsafe extern "C" fn snprintf(
    ptr: *mut c_char,
    len: usize,
    fmt: *const c_char,
    mut args: ...
) -> c_int {
    let va_list = args.as_va_list();
    vsnprintf(ptr, len, fmt, va_list)
}

#[no_mangle]
unsafe extern "C" fn vsnprintf(
    ptr: *mut c_char,
    len: usize,
    fmt: *const c_char,
    va_list: VaList,
) -> c_int {
    //libc!(libc::vsnprintf(ptr, len, fmt, va_list));

    let mut out = String::new();
    let num_bytes = format(fmt, va_list, output::fmt_write(&mut out));

    if num_bytes >= 0 {
        out.push('\0');

        copy_nonoverlapping(out.as_ptr(), ptr.cast(), min(num_bytes as usize + 1, len));
    }

    num_bytes
}

#[no_mangle]
unsafe extern "C" fn dprintf(fd: c_int, fmt: *const c_char, mut args: ...) -> c_int {
    let va_list = args.as_va_list();
    vdprintf(fd, fmt, va_list)
}

#[no_mangle]
unsafe extern "C" fn vdprintf(fd: c_int, fmt: *const c_char, va_list: VaList) -> c_int {
    //libc!(libc::vdprintf(fd, fmt, va_list));

    let mut tmp = std::fs::File::from_raw_fd(fd);
    let num_bytes = format(fmt, va_list, output::io_write(&mut tmp));
    let _ = tmp.into_raw_fd();
    num_bytes
}

#[no_mangle]
unsafe extern "C" fn fprintf(file: *mut c_void, fmt: *const c_char, mut args: ...) -> c_int {
    let va_list = args.as_va_list();
    vfprintf(file, fmt, va_list)
}

#[no_mangle]
unsafe extern "C" fn vfprintf(file: *mut c_void, fmt: *const c_char, va_list: VaList) -> c_int {
    //libc!(libc::vfprintf(file, fmt, va_list));

    if file == stdout {
        vprintf(fmt, va_list)
    } else if file == stderr {
        format(fmt, va_list, output::io_write(&mut rust_stderr()))
    } else {
        unimplemented!("vfprintf to a destination other than stdout or stderr")
    }
}

#[no_mangle]
unsafe extern "C" fn perror(user_message: *const c_char) {
    libc!(libc::perror(user_message));

    let errno_message = CStr::from_ptr(libc::strerror(errno::errno().0));
    let user_message = if user_message.is_null() {
        CStr::from_ptr(rustix::cstr!("").as_ptr())
    } else {
        CStr::from_ptr(user_message)
    };

    if user_message.to_bytes().is_empty() {
        eprintln!("{:?}", errno_message);
    } else {
        eprintln!("{:?}: {:?}", user_message, errno_message);
    }
}

#[no_mangle]
#[allow(non_upper_case_globals)]
static mut stdin: *mut c_void = unsafe { THE_STDIN.as_mut_ptr().cast() };
#[no_mangle]
#[allow(non_upper_case_globals)]
static mut stdout: *mut c_void = unsafe { THE_STDOUT.as_mut_ptr().cast() };
#[no_mangle]
#[allow(non_upper_case_globals)]
static mut stderr: *mut c_void = unsafe { THE_STDERR.as_mut_ptr().cast() };

static mut THE_STDIN: [u8; 1] = [0];
static mut THE_STDOUT: [u8; 1] = [1];
static mut THE_STDERR: [u8; 1] = [2];
