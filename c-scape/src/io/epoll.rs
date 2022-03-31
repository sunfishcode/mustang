// This entire module is marked
// #[cfg(any(target_os = "android", target_os = "linux"))]

use rustix::fd::{FromRawFd, IntoRawFd, OwnedFd};
use rustix::io::epoll::{CreateFlags, Epoll, Owning};

use errno::{set_errno, Errno};
use libc::c_int;

use crate::convert_res;

#[no_mangle]
unsafe extern "C" fn epoll_create(size: c_int) -> c_int {
    libc!(libc::epoll_create(size));

    if size <= 0 {
        set_errno(Errno(libc::EINVAL));
        return -1;
    }

    epoll_create1(0)
}

#[no_mangle]
unsafe extern "C" fn epoll_create1(flags: c_int) -> c_int {
    libc!(libc::epoll_create1(flags));

    let flags = CreateFlags::from_bits(flags as _).unwrap();

    match convert_res(Epoll::new(flags, Owning::<OwnedFd>::new())) {
        Some(epoll) => epoll.into_raw_fd(),
        None => -1,
    }
}

#[cfg(any(target_os = "android", target_os = "linux"))]
#[no_mangle]
unsafe extern "C" fn epoll_ctl(
    epfd: c_int,
    op: c_int,
    fd: c_int,
    event: *mut libc::epoll_event,
) -> c_int {
    let epoll = Epoll::from_raw_fd(epfd);

    // let res = match op {
    //     libc::EPOLL_CTL_ADD => epoll.add(),
    // };

    todo!()
}

#[no_mangle]
unsafe extern "C" fn epoll_wait(
    _epfd: c_int,
    _events: *mut libc::epoll_event,
    _maxevents: c_int,
    _timeout: c_int,
) -> c_int {
    libc!(libc::epoll_wait(
        _epfd,
        checked_cast!(_events),
        _maxevents,
        _timeout
    ));
    unimplemented!("epoll_wait")
}
