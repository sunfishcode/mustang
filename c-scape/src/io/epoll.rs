use rustix::fd::{BorrowedFd, IntoRawFd};
use rustix::io::epoll::{epoll_add, epoll_del, epoll_mod, CreateFlags, EventFlags, EventVec};

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

    match convert_res(rustix::io::epoll::epoll_create(flags)) {
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
    libc!(libc::epoll_ctl(epfd, op, fd, event));

    let epfd = BorrowedFd::borrow_raw(epfd);
    let fd = BorrowedFd::borrow_raw(fd);
    let res = match op {
        libc::EPOLL_CTL_ADD => {
            let libc::epoll_event { events, r#u64 } = event.read();
            let events = EventFlags::from_bits(events).unwrap();
            epoll_add(epfd, fd, r#u64, events)
        }
        libc::EPOLL_CTL_MOD => {
            let libc::epoll_event { events, r#u64 } = event.read();
            let events = EventFlags::from_bits(events).unwrap();
            epoll_mod(epfd, fd, r#u64, events)
        }
        libc::EPOLL_CTL_DEL => epoll_del(epfd, fd),
        _ => {
            set_errno(Errno(libc::EINVAL));
            return -1;
        }
    };

    match convert_res(res) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn epoll_wait(
    epfd: c_int,
    events: *mut libc::epoll_event,
    maxevents: c_int,
    timeout: c_int,
) -> c_int {
    libc!(libc::epoll_wait(epfd, events, maxevents, timeout));

    if maxevents <= 0 {
        set_errno(Errno(libc::EINVAL));
        return -1;
    }

    // TODO: We should add an `EventVec::from_raw_parts` to allow `epoll_wait`
    // to write events directly into the user's buffer, rather then allocating
    // and copying here.
    let mut events_vec = EventVec::with_capacity(maxevents as usize);
    match convert_res(rustix::io::epoll::epoll_wait(
        BorrowedFd::borrow_raw(epfd),
        &mut events_vec,
        timeout,
    )) {
        Some(()) => {
            let mut events = events;
            for (flags, data) in events_vec.iter() {
                events.write(libc::epoll_event {
                    events: flags.bits(),
                    r#u64: data,
                });
                events = events.add(1);
            }
            events_vec.len() as c_int
        }
        None => -1,
    }
}
