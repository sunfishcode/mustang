use core::ffi::CStr;
use core::mem::zeroed;
use core::ptr::null_mut;

use errno::{set_errno, Errno};
use libc::{c_char, c_int};
use rustix::net::{SocketAddrAny, SocketAddrStorage, SocketType};

use rustix::net::{IpAddr, SocketAddrV4, SocketAddrV6};
use sync_resolve::resolve_host;

extern crate alloc;

#[no_mangle]
unsafe extern "C" fn getaddrinfo(
    node: *const c_char,
    service: *const c_char,
    hints: *const libc::addrinfo,
    res: *mut *mut libc::addrinfo,
) -> c_int {
    libc!(libc::getaddrinfo(node, service, hints, res));

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

#[no_mangle]
unsafe extern "C" fn freeaddrinfo(mut res: *mut libc::addrinfo) {
    libc!(libc::freeaddrinfo(res));

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

#[no_mangle]
unsafe extern "C" fn __res_init() -> c_int {
    libc!(libc::res_init());
    0
}
