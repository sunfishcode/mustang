use rustix::fd::BorrowedFd;

use libc::c_int;

#[no_mangle]
unsafe extern "C" fn isatty(fd: c_int) -> c_int {
    libc!(libc::isatty(fd));

    // TODO: how does this handle the `EBADF`
    if rustix::io::isatty(&BorrowedFd::borrow_raw(fd)) {
        1
    } else {
        set_errno(Errno(libc::ENOTTY));
        0
    }
}
