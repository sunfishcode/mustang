// Constants and types used in `*-*-linux-gnu` targets. These are
// checked against `libc` by the parent mod.rs.

use std::ffi::c_void;
use std::os::raw::{c_int, c_long};

pub(crate) const F_SETFD: c_int = 2;
pub(crate) const F_GETFL: c_int = 3;
pub(crate) const F_DUPFD_CLOEXEC: c_int = 1030;

pub(crate) const SEEK_SET: c_int = 0;
pub(crate) const SEEK_CUR: c_int = 1;
pub(crate) const SEEK_END: c_int = 2;

pub(crate) const DT_UNKNOWN: u8 = 0;
pub(crate) const DT_FIFO: u8 = 1;
pub(crate) const DT_CHR: u8 = 2;
pub(crate) const DT_DIR: u8 = 4;
pub(crate) const DT_BLK: u8 = 6;
pub(crate) const DT_REG: u8 = 8;
pub(crate) const DT_LNK: u8 = 10;
pub(crate) const DT_SOCK: u8 = 12;

pub(crate) const TCGETS: c_long = 0x5401;
pub(crate) const FIONBIO: c_long = 0x5421;

pub(crate) const ALIGNOF_MAXALIGN_T: usize = 16;

pub(crate) const MAP_ANONYMOUS: i32 = 32;

pub(crate) const MAP_FAILED: *mut c_void = -1_isize as usize as *mut c_void;

pub(crate) const _SC_PAGESIZE: c_int = 30;
pub(crate) const _SC_GETPW_R_SIZE_MAX: c_int = 70;
pub(crate) const _SC_NPROCESSORS_ONLN: c_int = 84;
pub(crate) const _SC_SYMLOOP_MAX: c_int = 173;

pub(crate) const SYMLOOP_MAX: c_long = 40;

#[cfg(target_arch = "x86_64")]
pub(crate) const SYS_getrandom: c_long = 318;
#[cfg(target_arch = "x86")]
pub(crate) const SYS_getrandom: c_long = 355;
#[cfg(target_arch = "aarch64")]
pub(crate) const SYS_getrandom: c_long = 278;
#[cfg(target_arch = "riscv64")]
pub(crate) const SYS_getrandom: c_long = 278;

pub(crate) const CLOCK_REALTIME: c_int = 0;
pub(crate) const CLOCK_MONOTONIC: c_int = 1;

pub(crate) const SIG_DFL: usize = 0;

pub(crate) const SHUT_RD: c_int = 0;
pub(crate) const SHUT_WR: c_int = 1;
pub(crate) const SHUT_RDWR: c_int = 2;

pub(crate) const AF_UNIX: c_int = 1;
pub(crate) const AF_INET: c_int = 2;
pub(crate) const AF_INET6: c_int = 10;
pub(crate) const AF_NETLINK: c_int = 16;

pub(crate) const SOCK_STREAM: c_int = 1;
pub(crate) const SOCK_DGRAM: c_int = 2;
pub(crate) const SOCK_RAW: c_int = 3;
pub(crate) const SOCK_RDM: c_int = 4;
pub(crate) const SOCK_SEQPACKET: c_int = 5;

pub(crate) const IPPROTO_IP: c_int = 0;
pub(crate) const IPPROTO_ICMP: c_int = 1;
pub(crate) const IPPROTO_IGMP: c_int = 2;
pub(crate) const IPPROTO_IPIP: c_int = 4;
pub(crate) const IPPROTO_TCP: c_int = 6;
pub(crate) const IPPROTO_EGP: c_int = 8;
pub(crate) const IPPROTO_PUP: c_int = 12;
pub(crate) const IPPROTO_UDP: c_int = 17;
pub(crate) const IPPROTO_IDP: c_int = 22;
pub(crate) const IPPROTO_TP: c_int = 29;
pub(crate) const IPPROTO_DCCP: c_int = 33;
pub(crate) const IPPROTO_IPV6: c_int = 41;
pub(crate) const IPPROTO_RSVP: c_int = 46;
pub(crate) const IPPROTO_GRE: c_int = 47;
pub(crate) const IPPROTO_ESP: c_int = 50;
pub(crate) const IPPROTO_AH: c_int = 51;
pub(crate) const IPPROTO_MTP: c_int = 92;
pub(crate) const IPPROTO_BEETPH: c_int = 94;
pub(crate) const IPPROTO_ENCAP: c_int = 98;
pub(crate) const IPPROTO_PIM: c_int = 103;
pub(crate) const IPPROTO_COMP: c_int = 108;
pub(crate) const IPPROTO_SCTP: c_int = 132;
pub(crate) const IPPROTO_UDPLITE: c_int = 136;
pub(crate) const IPPROTO_MPLS: c_int = 137;
pub(crate) const IPPROTO_ETHERNET: c_int = 143;
pub(crate) const IPPROTO_RAW: c_int = 255;
pub(crate) const IPPROTO_MPTCP: c_int = 262;

#[repr(C)]
pub(crate) struct Dirent64 {
    pub(crate) d_ino: u64,
    pub(crate) d_off: u64,
    pub(crate) d_reclen: u16,
    pub(crate) d_type: u8,
    pub(crate) d_name: [u8; 256],
}

pub(crate) type SockLen = u32;

/* FIXME: implement getsockopt, setsockopt, getaddrinfo, and freeaddrinfo
#[repr(C)]
pub(crate) struct Addrinfo {
    ai_flags: c_int,
    ai_family: c_int,
    ai_socktype: c_int,
    ai_protocol: c_int,
    ai_addrlen: SockLen,
    ai_addr: *mut rsix::net::SocketAddrStorage,
    ai_canonname: *mut i8,
    ai_next: *mut Addrinfo,
}
*/
