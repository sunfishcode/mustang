use rustix::fd::BorrowedFd;

use errno::{set_errno, Errno};
use libc::{c_int, off64_t, off_t, size_t};

use crate::convert_res;

#[cfg(any(target_os = "android", target_os = "linux"))]
#[no_mangle]
unsafe extern "C" fn sendfile(
    out_fd: c_int,
    in_fd: c_int,
    offset: *mut off_t,
    count: size_t,
) -> isize {
    libc!(libc::sendfile(out_fd, in_fd, offset, count));

    // Check for overflow into off64_t
    if !offset.is_null() {
        if *offset < 0 {
            set_errno(Errno(libc::EINVAL));
            return -1;
        }

        // If count is too big then we just fail immediately
        if let Ok(add) = TryInto::<off_t>::try_into(count) {
            // Check if count + offset could overflow and return EINVAL
            if add.overflowing_add(*offset).1 {
                set_errno(Errno(libc::EINVAL));
                return -1;
            }
        } else {
            set_errno(Errno(libc::EINVAL));
            return -1;
        }
    }

    let mut offset_64: off64_t = 0;
    let res = sendfile64(out_fd, in_fd, &mut offset_64, count);

    if !offset.is_null() {
        // Safe cast as we checked above
        *offset = offset_64 as off_t;
    }

    res
}

#[cfg(any(target_os = "android", target_os = "linux"))]
#[no_mangle]
unsafe extern "C" fn sendfile64(
    out_fd: c_int,
    in_fd: c_int,
    offset: *mut off64_t,
    count: size_t,
) -> isize {
    libc!(libc::sendfile64(out_fd, in_fd, offset, count));

    let offset: *mut u64 = checked_cast!(offset);

    match convert_res(rustix::fs::sendfile(
        BorrowedFd::borrow_raw(out_fd),
        BorrowedFd::borrow_raw(in_fd),
        offset.as_mut(),
        count.into(),
    )) {
        Some(num) => num.try_into().unwrap(),
        None => -1,
    }
}
