use rustix::fd::BorrowedFd;

use libc::c_int;

use crate::convert_res;

#[no_mangle]
unsafe extern "C" fn isatty(fd: c_int) -> c_int {
    libc!(libc::isatty(fd));

    match convert_res(rustix::io::ioctl_tiocgwinsz(&BorrowedFd::borrow_raw(fd))) {
        Some(_) => 1,
        None => 0,
    }
}
