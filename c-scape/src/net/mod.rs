mod inet;

#[cfg(feature = "sync-resolve")]
use alloc::string::ToString;
use core::convert::TryInto;
use core::ffi::{c_void, CStr};
#[cfg(not(target_os = "wasi"))]
use core::mem::{size_of, zeroed};
use core::ptr::null_mut;
use core::slice;
use errno::{set_errno, Errno};
use libc::{c_char, c_int, c_uint};
use rustix::fd::{BorrowedFd, IntoRawFd};
#[cfg(feature = "net")]
use rustix::net::{
    AcceptFlags, AddressFamily, Ipv4Addr, Ipv6Addr, Protocol, RecvFlags, SendFlags, Shutdown,
    SocketAddrAny, SocketAddrStorage, SocketFlags, SocketType,
};
#[cfg(feature = "sync-resolve")]
use {
    rustix::net::{IpAddr, SocketAddrV4, SocketAddrV6},
    sync_resolve::resolve_host,
};

use crate::convert_res;

#[no_mangle]
unsafe extern "C" fn accept(
    fd: c_int,
    addr: *mut SocketAddrStorage,
    len: *mut libc::socklen_t,
) -> c_int {
    // We don't use `checked_cast` here because libc uses `sockaddr` which
    // just represents the header of the struct, not the full storage.
    libc!(libc::accept(fd, addr.cast(), len));

    match convert_res(rustix::net::acceptfrom(BorrowedFd::borrow_raw(fd))) {
        Some((accepted_fd, from)) => {
            if let Some(from) = from {
                let encoded_len = from.write(addr);
                *len = encoded_len.try_into().unwrap();
            }
            accepted_fd.into_raw_fd()
        }
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn accept4(
    fd: c_int,
    addr: *mut SocketAddrStorage,
    len: *mut libc::socklen_t,
    flags: c_int,
) -> c_int {
    // We don't use `checked_cast` here because libc uses `sockaddr` which
    // just represents the header of the struct, not the full storage.
    libc!(libc::accept4(fd, addr.cast(), len, flags));

    let flags = AcceptFlags::from_bits(flags as _).unwrap();
    match convert_res(rustix::net::acceptfrom_with(
        BorrowedFd::borrow_raw(fd),
        flags,
    )) {
        Some((accepted_fd, from)) => {
            if let Some(from) = from {
                let encoded_len = from.write(addr);
                *len = encoded_len.try_into().unwrap();
            }
            accepted_fd.into_raw_fd()
        }
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn bind(
    sockfd: c_int,
    addr: *const SocketAddrStorage,
    len: libc::socklen_t,
) -> c_int {
    // We don't use `checked_cast` here because libc uses `sockaddr` which
    // just represents the header of the struct, not the full storage.
    libc!(libc::bind(sockfd, addr.cast(), len));

    let addr = match convert_res(SocketAddrAny::read(addr, len.try_into().unwrap())) {
        Some(addr) => addr,
        None => return -1,
    };
    match convert_res(match addr {
        SocketAddrAny::V4(v4) => rustix::net::bind_v4(BorrowedFd::borrow_raw(sockfd), &v4),
        SocketAddrAny::V6(v6) => rustix::net::bind_v6(BorrowedFd::borrow_raw(sockfd), &v6),
        SocketAddrAny::Unix(unix) => rustix::net::bind_unix(BorrowedFd::borrow_raw(sockfd), &unix),
        _ => panic!("unrecognized SocketAddrAny kind"),
    }) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn connect(
    sockfd: c_int,
    addr: *const SocketAddrStorage,
    len: libc::socklen_t,
) -> c_int {
    // We don't use `checked_cast` here because libc uses `sockaddr` which
    // just represents the header of the struct, not the full storage.
    libc!(libc::connect(sockfd, addr.cast(), len));

    let addr = match convert_res(SocketAddrAny::read(addr, len.try_into().unwrap())) {
        Some(addr) => addr,
        None => return -1,
    };
    match convert_res(match addr {
        SocketAddrAny::V4(v4) => rustix::net::connect_v4(BorrowedFd::borrow_raw(sockfd), &v4),
        SocketAddrAny::V6(v6) => rustix::net::connect_v6(BorrowedFd::borrow_raw(sockfd), &v6),
        SocketAddrAny::Unix(unix) => {
            rustix::net::connect_unix(BorrowedFd::borrow_raw(sockfd), &unix)
        }
        _ => panic!("unrecognized SocketAddrAny kind"),
    }) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn getpeername(
    fd: c_int,
    addr: *mut SocketAddrStorage,
    len: *mut libc::socklen_t,
) -> c_int {
    // We don't use `checked_cast` here because libc uses `sockaddr` which
    // just represents the header of the struct, not the full storage.
    libc!(libc::getpeername(fd, addr.cast(), len));

    match convert_res(rustix::net::getpeername(BorrowedFd::borrow_raw(fd))) {
        Some(from) => {
            if let Some(from) = from {
                let encoded_len = from.write(addr);
                *len = encoded_len.try_into().unwrap();
            }
            0
        }
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn getsockname(
    fd: c_int,
    addr: *mut SocketAddrStorage,
    len: *mut libc::socklen_t,
) -> c_int {
    // We don't use `checked_cast` here because libc uses `sockaddr` which
    // just represents the header of the struct, not the full storage.
    libc!(libc::getsockname(fd, addr.cast(), len));

    match convert_res(rustix::net::getsockname(BorrowedFd::borrow_raw(fd))) {
        Some(from) => {
            let encoded_len = from.write(addr);
            *len = encoded_len.try_into().unwrap();
            0
        }
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn getsockopt(
    fd: c_int,
    level: c_int,
    optname: c_int,
    optval: *mut c_void,
    optlen: *mut libc::socklen_t,
) -> c_int {
    use core::time::Duration;
    use rustix::net::sockopt::{self, Timeout};

    unsafe fn write_bool(
        value: rustix::io::Result<bool>,
        optval: *mut c_void,
        optlen: *mut libc::socklen_t,
    ) -> rustix::io::Result<()> {
        Ok(write(value? as c_uint, optval.cast::<c_uint>(), optlen))
    }

    unsafe fn write_u32(
        value: rustix::io::Result<u32>,
        optval: *mut c_void,
        optlen: *mut libc::socklen_t,
    ) -> rustix::io::Result<()> {
        Ok(write(value?, optval.cast::<u32>(), optlen))
    }

    unsafe fn write_linger(
        linger: rustix::io::Result<Option<Duration>>,
        optval: *mut c_void,
        optlen: *mut libc::socklen_t,
    ) -> rustix::io::Result<()> {
        let linger = linger?;
        let linger = libc::linger {
            l_onoff: linger.is_some() as c_int,
            l_linger: linger.unwrap_or_default().as_secs() as c_int,
        };
        Ok(write(linger, optval.cast::<libc::linger>(), optlen))
    }

    unsafe fn write_timeval(
        value: rustix::io::Result<Option<Duration>>,
        optval: *mut c_void,
        optlen: *mut libc::socklen_t,
    ) -> rustix::io::Result<()> {
        let timeval = match value? {
            None => libc::timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            Some(duration) => libc::timeval {
                tv_sec: duration
                    .as_secs()
                    .try_into()
                    .map_err(|_| rustix::io::Errno::OVERFLOW)?,
                tv_usec: duration.subsec_micros() as _,
            },
        };
        Ok(write(timeval, optval.cast::<libc::timeval>(), optlen))
    }

    unsafe fn write<T>(value: T, optval: *mut T, optlen: *mut libc::socklen_t) {
        *optlen = size_of::<T>().try_into().unwrap();
        optval.write(value)
    }

    libc!(libc::getsockopt(fd, level, optname, optval, optlen));

    let fd = BorrowedFd::borrow_raw(fd);
    let result = match level {
        libc::SOL_SOCKET => match optname {
            libc::SO_BROADCAST => write_bool(sockopt::get_socket_broadcast(fd), optval, optlen),
            libc::SO_LINGER => write_linger(sockopt::get_socket_linger(fd), optval, optlen),
            libc::SO_PASSCRED => write_bool(sockopt::get_socket_passcred(fd), optval, optlen),
            libc::SO_SNDTIMEO => write_timeval(
                sockopt::get_socket_timeout(fd, Timeout::Send),
                optval,
                optlen,
            ),
            libc::SO_RCVTIMEO => write_timeval(
                sockopt::get_socket_timeout(fd, Timeout::Recv),
                optval,
                optlen,
            ),
            _ => unimplemented!("unimplemented getsockopt SOL_SOCKET optname {:?}", optname),
        },
        libc::IPPROTO_IP => match optname {
            libc::IP_TTL => write_u32(sockopt::get_ip_ttl(fd), optval, optlen),
            libc::IP_MULTICAST_LOOP => {
                write_bool(sockopt::get_ip_multicast_loop(fd), optval, optlen)
            }
            libc::IP_MULTICAST_TTL => write_u32(sockopt::get_ip_multicast_ttl(fd), optval, optlen),
            _ => unimplemented!("unimplemented getsockopt IPPROTO_IP optname {:?}", optname),
        },
        libc::IPPROTO_IPV6 => match optname {
            libc::IPV6_MULTICAST_LOOP => {
                write_bool(sockopt::get_ipv6_multicast_loop(fd), optval, optlen)
            }
            libc::IPV6_V6ONLY => write_bool(sockopt::get_ipv6_v6only(fd), optval, optlen),
            _ => unimplemented!(
                "unimplemented getsockopt IPPROTO_IPV6 optname {:?}",
                optname
            ),
        },
        libc::IPPROTO_TCP => match optname {
            libc::TCP_NODELAY => write_bool(sockopt::get_tcp_nodelay(fd), optval, optlen),
            _ => unimplemented!("unimplemented getsockopt IPPROTO_TCP optname {:?}", optname),
        },
        _ => unimplemented!(
            "unimplemented getsockopt level {:?} optname {:?}",
            level,
            optname
        ),
    };
    match convert_res(result) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn setsockopt(
    fd: c_int,
    level: c_int,
    optname: c_int,
    optval: *const c_void,
    optlen: libc::socklen_t,
) -> c_int {
    use core::time::Duration;
    use rustix::net::sockopt::{self, Timeout};

    unsafe fn read_bool(optval: *const c_void, optlen: libc::socklen_t) -> bool {
        read(optval.cast::<c_int>(), optlen) != 0
    }

    unsafe fn read_u32(optval: *const c_void, optlen: libc::socklen_t) -> u32 {
        read(optval.cast::<u32>(), optlen)
    }

    unsafe fn read_linger(optval: *const c_void, optlen: libc::socklen_t) -> Option<Duration> {
        let linger = read(optval.cast::<libc::linger>(), optlen);
        (linger.l_onoff != 0).then(|| Duration::from_secs(linger.l_linger as u64))
    }

    unsafe fn read_timeval(optval: *const c_void, optlen: libc::socklen_t) -> Option<Duration> {
        let timeval = read(optval.cast::<libc::timeval>(), optlen);
        if timeval.tv_sec == 0 && timeval.tv_usec == 0 {
            None
        } else {
            Some(
                Duration::from_secs(timeval.tv_sec.try_into().unwrap())
                    + Duration::from_micros(timeval.tv_usec as _),
            )
        }
    }

    unsafe fn read_ip_multiaddr(optval: *const c_void, optlen: libc::socklen_t) -> Ipv4Addr {
        Ipv4Addr::from(
            read(optval.cast::<libc::ip_mreq>(), optlen)
                .imr_multiaddr
                .s_addr,
        )
    }

    unsafe fn read_ip_interface(optval: *const c_void, optlen: libc::socklen_t) -> Ipv4Addr {
        Ipv4Addr::from(
            read(optval.cast::<libc::ip_mreq>(), optlen)
                .imr_interface
                .s_addr,
        )
    }

    unsafe fn read_ipv6_multiaddr(optval: *const c_void, optlen: libc::socklen_t) -> Ipv6Addr {
        Ipv6Addr::from(
            read(optval.cast::<libc::ipv6_mreq>(), optlen)
                .ipv6mr_multiaddr
                .s6_addr,
        )
    }

    unsafe fn read_ipv6_interface(optval: *const c_void, optlen: libc::socklen_t) -> u32 {
        read(optval.cast::<libc::ipv6_mreq>(), optlen).ipv6mr_interface
    }

    unsafe fn read<T>(optval: *const T, optlen: libc::socklen_t) -> T {
        assert_eq!(optlen, size_of::<T>().try_into().unwrap());
        optval.read()
    }

    libc!(libc::setsockopt(fd, level, optname, optval, optlen));

    let fd = BorrowedFd::borrow_raw(fd);
    let result = match level {
        libc::SOL_SOCKET => match optname {
            libc::SO_REUSEADDR => sockopt::set_socket_reuseaddr(fd, read_bool(optval, optlen)),
            libc::SO_BROADCAST => sockopt::set_socket_broadcast(fd, read_bool(optval, optlen)),
            libc::SO_LINGER => sockopt::set_socket_linger(fd, read_linger(optval, optlen)),
            libc::SO_PASSCRED => sockopt::set_socket_passcred(fd, read_bool(optval, optlen)),
            libc::SO_SNDTIMEO => {
                sockopt::set_socket_timeout(fd, Timeout::Send, read_timeval(optval, optlen))
            }
            libc::SO_RCVTIMEO => {
                sockopt::set_socket_timeout(fd, Timeout::Recv, read_timeval(optval, optlen))
            }
            _ => unimplemented!("unimplemented setsockopt SOL_SOCKET optname {:?}", optname),
        },
        libc::IPPROTO_IP => match optname {
            libc::IP_TTL => sockopt::set_ip_ttl(fd, read_u32(optval, optlen)),
            libc::IP_MULTICAST_LOOP => {
                sockopt::set_ip_multicast_loop(fd, read_bool(optval, optlen))
            }
            libc::IP_MULTICAST_TTL => sockopt::set_ip_multicast_ttl(fd, read_u32(optval, optlen)),
            libc::IP_ADD_MEMBERSHIP => sockopt::set_ip_add_membership(
                fd,
                &read_ip_multiaddr(optval, optlen),
                &read_ip_interface(optval, optlen),
            ),
            libc::IP_DROP_MEMBERSHIP => sockopt::set_ip_add_membership(
                fd,
                &read_ip_multiaddr(optval, optlen),
                &read_ip_interface(optval, optlen),
            ),
            _ => unimplemented!("unimplemented setsockopt IPPROTO_IP optname {:?}", optname),
        },
        libc::IPPROTO_IPV6 => match optname {
            libc::IPV6_MULTICAST_LOOP => {
                sockopt::set_ipv6_multicast_loop(fd, read_bool(optval, optlen))
            }
            libc::IPV6_ADD_MEMBERSHIP => sockopt::set_ipv6_add_membership(
                fd,
                &read_ipv6_multiaddr(optval, optlen),
                read_ipv6_interface(optval, optlen),
            ),
            libc::IPV6_DROP_MEMBERSHIP => sockopt::set_ipv6_drop_membership(
                fd,
                &read_ipv6_multiaddr(optval, optlen),
                read_ipv6_interface(optval, optlen),
            ),
            libc::IPV6_V6ONLY => sockopt::set_ipv6_v6only(fd, read_bool(optval, optlen)),
            _ => unimplemented!(
                "unimplemented setsockopt IPPROTO_IPV6 optname {:?}",
                optname
            ),
        },
        libc::IPPROTO_TCP => match optname {
            libc::TCP_NODELAY => sockopt::set_tcp_nodelay(fd, read_bool(optval, optlen)),
            _ => unimplemented!("unimplemented setsockopt IPPROTO_TCP optname {:?}", optname),
        },
        _ => unimplemented!(
            "unimplemented setsockopt level {:?} optname {:?}",
            level,
            optname
        ),
    };
    match convert_res(result) {
        Some(()) => 0,
        None => -1,
    }
}

#[cfg(feature = "sync-resolve")]
#[no_mangle]
unsafe extern "C" fn getaddrinfo(
    node: *const c_char,
    service: *const c_char,
    hints: *const libc::addrinfo,
    res: *mut *mut libc::addrinfo,
) -> c_int {
    libc!(libc::getaddrinfo(
        node,
        service,
        checked_cast!(hints),
        checked_cast!(res)
    ));

    assert!(service.is_null(), "service lookups not supported yet");
    assert!(!node.is_null(), "only name lookups are supported corrently");

    if !hints.is_null() {
        let hints = &*hints;
        assert_eq!(hints.ai_flags, 0, "GAI flags hint not supported yet");
        assert_eq!(hints.ai_family, 0, "GAI family hint not supported yet");
        assert_eq!(
            hints.ai_socktype,
            SocketType::STREAM.as_raw() as _,
            "only SOCK_STREAM supported currently"
        );
        assert_eq!(hints.ai_protocol, 0, "GAI protocl hint not supported yet");
        assert_eq!(hints.ai_addrlen, 0, "GAI addrlen hint not supported yet");
        assert!(hints.ai_addr.is_null(), "GAI addr hint not supported yet");
        assert!(
            hints.ai_canonname.is_null(),
            "GAI canonname hint not supported yet"
        );
        assert!(hints.ai_next.is_null(), "GAI next hint not supported yet");
    }

    let host = match CStr::from_ptr(node.cast()).to_str() {
        Ok(host) => host,
        Err(_) => {
            set_errno(Errno(libc::EILSEQ));
            return libc::EAI_SYSTEM;
        }
    };

    let layout = alloc::alloc::Layout::new::<libc::addrinfo>();
    let addr_layout = alloc::alloc::Layout::new::<SocketAddrStorage>();
    let mut first: *mut libc::addrinfo = null_mut();
    let mut prev: *mut libc::addrinfo = null_mut();
    match resolve_host(host) {
        Ok(addrs) => {
            for addr in addrs {
                let ptr = alloc::alloc::alloc(layout).cast::<libc::addrinfo>();
                ptr.write(zeroed());
                let info = &mut *ptr;
                match addr {
                    IpAddr::V4(v4) => {
                        // TODO: Create and write to `SocketAddrV4Storage`?
                        let storage = alloc::alloc::alloc(addr_layout).cast::<SocketAddrStorage>();
                        let len = SocketAddrAny::V4(SocketAddrV4::new(v4, 0)).write(storage);
                        info.ai_addr = storage.cast();
                        info.ai_addrlen = len.try_into().unwrap();
                    }
                    IpAddr::V6(v6) => {
                        // TODO: Create and write to `SocketAddrV6Storage`?
                        let storage = alloc::alloc::alloc(addr_layout).cast::<SocketAddrStorage>();
                        let len = SocketAddrAny::V6(SocketAddrV6::new(v6, 0, 0, 0)).write(storage);
                        info.ai_addr = storage.cast();
                        info.ai_addrlen = len.try_into().unwrap();
                    }
                }
                if !prev.is_null() {
                    (*prev).ai_next = ptr;
                }
                prev = ptr;
                if first.is_null() {
                    first = ptr;
                }
            }
            *res = first;
            0
        }
        Err(err) => {
            if let Some(err) = rustix::io::Errno::from_io_error(&err) {
                set_errno(Errno(err.raw_os_error()));
                libc::EAI_SYSTEM
            } else {
                // TODO: sync-resolve should return a custom error type
                if err.to_string()
                    == "failed to resolve host: server responded with error: server failure"
                    || err.to_string()
                        == "failed to resolve host: server responded with error: no such name"
                {
                    libc::EAI_NONAME
                } else {
                    panic!("unknown error: {}", err);
                }
            }
        }
    }
}

#[cfg(feature = "sync-resolve")]
#[no_mangle]
unsafe extern "C" fn freeaddrinfo(mut res: *mut libc::addrinfo) {
    libc!(libc::freeaddrinfo(checked_cast!(res)));

    let layout = alloc::alloc::Layout::new::<libc::addrinfo>();
    let addr_layout = alloc::alloc::Layout::new::<SocketAddrStorage>();

    while !res.is_null() {
        let addr = (*res).ai_addr;
        if !addr.is_null() {
            alloc::alloc::dealloc(addr.cast(), addr_layout);
        }
        let old = res;
        res = (*res).ai_next;
        alloc::alloc::dealloc(old.cast(), layout);
    }
}

#[cfg(feature = "sync-resolve")]
#[no_mangle]
unsafe extern "C" fn gai_strerror(errcode: c_int) -> *const c_char {
    libc!(libc::gai_strerror(errcode));

    match errcode {
        libc::EAI_NONAME => &b"Name does not resolve\0"[..],
        libc::EAI_SYSTEM => &b"System error\0"[..],
        _ => panic!("unrecognized gai_strerror {:?}", errcode),
    }
    .as_ptr()
    .cast()
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn gethostname(name: *mut c_char, len: usize) -> c_int {
    let uname = rustix::process::uname();
    let nodename = uname.nodename();
    if nodename.to_bytes().len() + 1 > len {
        set_errno(Errno(libc::ENAMETOOLONG));
        return -1;
    }
    libc::memcpy(
        name.cast(),
        nodename.to_bytes().as_ptr().cast(),
        nodename.to_bytes().len(),
    );
    *name.add(nodename.to_bytes().len()) = 0;
    0
}

#[no_mangle]
unsafe extern "C" fn listen(fd: c_int, backlog: c_int) -> c_int {
    libc!(libc::listen(fd, backlog));

    match convert_res(rustix::net::listen(BorrowedFd::borrow_raw(fd), backlog)) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn recv(fd: c_int, ptr: *mut c_void, len: usize, flags: c_int) -> isize {
    libc!(libc::recv(fd, ptr, len, flags));

    let flags = RecvFlags::from_bits(flags as _).unwrap();
    match convert_res(rustix::net::recv(
        BorrowedFd::borrow_raw(fd),
        slice::from_raw_parts_mut(ptr.cast::<u8>(), len),
        flags,
    )) {
        Some(nread) => nread as isize,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn recvfrom(
    fd: c_int,
    buf: *mut c_void,
    len: usize,
    flags: c_int,
    from: *mut SocketAddrStorage,
    from_len: *mut libc::socklen_t,
) -> isize {
    // We don't use `checked_cast` here because libc uses `sockaddr` which
    // just represents the header of the struct, not the full storage.
    libc!(libc::recvfrom(fd, buf, len, flags, from.cast(), from_len));

    let flags = RecvFlags::from_bits(flags as _).unwrap();
    match convert_res(rustix::net::recvfrom(
        BorrowedFd::borrow_raw(fd),
        slice::from_raw_parts_mut(buf.cast::<u8>(), len),
        flags,
    )) {
        Some((nread, addr)) => {
            if let Some(addr) = addr {
                let encoded_len = addr.write(from);
                *from_len = encoded_len.try_into().unwrap();
            }
            nread as isize
        }
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn send(fd: c_int, buf: *const c_void, len: usize, flags: c_int) -> isize {
    libc!(libc::send(fd, buf, len, flags));

    let flags = SendFlags::from_bits(flags as _).unwrap();
    match convert_res(rustix::net::send(
        BorrowedFd::borrow_raw(fd),
        slice::from_raw_parts(buf.cast::<u8>(), len),
        flags,
    )) {
        Some(nwritten) => nwritten as isize,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn sendto(
    fd: c_int,
    buf: *const c_void,
    len: usize,
    flags: c_int,
    to: *const SocketAddrStorage,
    to_len: libc::socklen_t,
) -> isize {
    // We don't use `checked_cast` here because libc uses `sockaddr` which
    // just represents the header of the struct, not the full storage.
    libc!(libc::sendto(fd, buf, len, flags, to.cast(), to_len));

    let flags = SendFlags::from_bits(flags as _).unwrap();
    let addr = match convert_res(SocketAddrAny::read(to, to_len.try_into().unwrap())) {
        Some(addr) => addr,
        None => return -1,
    };
    match convert_res(match addr {
        SocketAddrAny::V4(v4) => rustix::net::sendto_v4(
            BorrowedFd::borrow_raw(fd),
            slice::from_raw_parts(buf.cast::<u8>(), len),
            flags,
            &v4,
        ),
        SocketAddrAny::V6(v6) => rustix::net::sendto_v6(
            BorrowedFd::borrow_raw(fd),
            slice::from_raw_parts(buf.cast::<u8>(), len),
            flags,
            &v6,
        ),
        SocketAddrAny::Unix(unix) => rustix::net::sendto_unix(
            BorrowedFd::borrow_raw(fd),
            slice::from_raw_parts(buf.cast::<u8>(), len),
            flags,
            &unix,
        ),
        _ => panic!("unrecognized SocketAddrAny kind"),
    }) {
        Some(nwritten) => nwritten as isize,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn shutdown(fd: c_int, how: c_int) -> c_int {
    libc!(libc::shutdown(fd, how));

    let how = match how {
        libc::SHUT_RD => Shutdown::Read,
        libc::SHUT_WR => Shutdown::Write,
        libc::SHUT_RDWR => Shutdown::ReadWrite,
        _ => panic!("unrecognized shutdown kind {}", how),
    };
    match convert_res(rustix::net::shutdown(BorrowedFd::borrow_raw(fd), how)) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn socket(domain: c_int, type_: c_int, protocol: c_int) -> c_int {
    libc!(libc::socket(domain, type_, protocol));

    let domain = AddressFamily::from_raw(domain as _);
    let flags = SocketFlags::from_bits_truncate(type_ as _);
    let type_ = SocketType::from_raw(type_ as u32 & !SocketFlags::all().bits());
    let protocol = Protocol::from_raw(protocol as _);
    match convert_res(rustix::net::socket_with(domain, type_, flags, protocol)) {
        Some(fd) => fd.into_raw_fd(),
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn socketpair(
    domain: c_int,
    type_: c_int,
    protocol: c_int,
    sv: *mut [c_int; 2],
) -> c_int {
    libc!(libc::socketpair(
        domain,
        type_,
        protocol,
        (*sv).as_mut_ptr()
    ));

    let domain = AddressFamily::from_raw(domain as _);
    let flags = SocketFlags::from_bits_truncate(type_ as _);
    let type_ = SocketType::from_raw(type_ as u32 & !SocketFlags::all().bits());
    let protocol = Protocol::from_raw(protocol as _);
    match convert_res(rustix::net::socketpair(domain, type_, flags, protocol)) {
        Some((fd0, fd1)) => {
            (*sv) = [fd0.into_raw_fd(), fd1.into_raw_fd()];
            0
        }
        None => -1,
    }
}

#[cfg(feature = "sync-resolve")]
#[no_mangle]
unsafe extern "C" fn __res_init() -> c_int {
    libc!(libc::res_init());
    0
}
