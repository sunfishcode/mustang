use rustix::fd::BorrowedFd;
use rustix::fs::Advice;

use libc::{c_int, off_t};

use crate::convert_res;

#[no_mangle]
unsafe extern "C" fn posix_fadvise(fd: c_int, offset: off_t, len: off_t, advice: c_int) -> c_int {
    libc!(libc::posix_fadvise(fd, offset, len, advice));

    let advice = match advice {
        libc::POSIX_FADV_NORMAL => Advice::Normal,
        libc::POSIX_FADV_SEQUENTIAL => Advice::Sequential,
        libc::POSIX_FADV_RANDOM => Advice::Random,
        libc::POSIX_FADV_NOREUSE => Advice::NoReuse,
        libc::POSIX_FADV_WILLNEED => Advice::WillNeed,
        libc::POSIX_FADV_DONTNEED => Advice::DontNeed,
        _ => panic!(),
    };
    match convert_res(rustix::fs::fadvise(
        BorrowedFd::borrow_raw(fd),
        offset as u64,
        len as u64,
        advice,
    )) {
        Some(()) => 0,
        None => -1,
    }
}
