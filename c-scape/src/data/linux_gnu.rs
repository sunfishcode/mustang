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
