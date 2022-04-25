use core::convert::TryInto;
use core::slice;
use libc::c_int;

use crate::convert_res;

#[no_mangle]
unsafe extern "C" fn poll(fds: *mut libc::pollfd, nfds: libc::nfds_t, timeout: c_int) -> c_int {
    libc!(libc::poll(fds, nfds, timeout));

    let pollfds: *mut rustix::io::PollFd = checked_cast!(fds);

    let fds = slice::from_raw_parts_mut(pollfds, nfds.try_into().unwrap());
    match convert_res(rustix::io::poll(fds, timeout)) {
        Some(num) => num.try_into().unwrap(),
        None => -1,
    }
}
