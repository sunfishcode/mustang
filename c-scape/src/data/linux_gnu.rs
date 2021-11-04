// Constants and types used in `*-*-linux-gnu` targets. These are
// checked against `libc` by the parent mod.rs.

#![allow(dead_code)]

use std::ffi::c_void;
use std::os::raw::{c_char, c_int, c_long, c_ulong, c_ulonglong};

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
pub(crate) const MAP_FAILED: *mut c_void = !0_usize as *mut c_void;
pub(crate) const MREMAP_MAYMOVE: i32 = 1;
pub(crate) const MREMAP_FIXED: i32 = 2;
pub(crate) const MREMAP_DONTUNMAP: i32 = 4;

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
#[cfg(target_arch = "arm")]
pub(crate) const SYS_getrandom: c_long = 384;

#[cfg(target_arch = "x86_64")]
pub(crate) const SYS_futex: c_long = 202;
#[cfg(target_arch = "x86")]
pub(crate) const SYS_futex: c_long = 240;
#[cfg(target_arch = "aarch64")]
pub(crate) const SYS_futex: c_long = 98;
#[cfg(target_arch = "riscv64")]
pub(crate) const SYS_futex: c_long = 98;
#[cfg(target_arch = "arm")]
pub(crate) const SYS_futex: c_long = 240;

pub(crate) const CLOCK_REALTIME: c_int = 0;
pub(crate) const CLOCK_MONOTONIC: c_int = 1;

pub(crate) const SIG_DFL: usize = 0;

pub(crate) const SHUT_RD: c_int = 0;
pub(crate) const SHUT_WR: c_int = 1;
pub(crate) const SHUT_RDWR: c_int = 2;

// `getsockopt`/`setsockopt` options.

pub(crate) const SOL_SOCKET: c_int = 1;

pub(crate) const IPPROTO_IP: c_int = 0;
pub(crate) const IPPROTO_TCP: c_int = 6;
pub(crate) const IPPROTO_IPV6: c_int = 41;

pub(crate) const SO_DEBUG: c_int = 1;
pub(crate) const SO_REUSEADDR: c_int = 2;
pub(crate) const SO_TYPE: c_int = 3;
pub(crate) const SO_ERROR: c_int = 4;
pub(crate) const SO_DONTROUTE: c_int = 5;
pub(crate) const SO_BROADCAST: c_int = 6;
pub(crate) const SO_SNDBUF: c_int = 7;
pub(crate) const SO_RCVBUF: c_int = 8;
pub(crate) const SO_KEEPALIVE: c_int = 9;
pub(crate) const SO_OOBINLINE: c_int = 10;
pub(crate) const SO_NO_CHECK: c_int = 11;
pub(crate) const SO_PRIORITY: c_int = 12;
pub(crate) const SO_LINGER: c_int = 13;
pub(crate) const SO_BSDCOMPAT: c_int = 14;
pub(crate) const SO_REUSEPORT: c_int = 15;
pub(crate) const SO_PASSCRED: c_int = 16;
pub(crate) const SO_PEERCRED: c_int = 17;
pub(crate) const SO_RCVLOWAT: c_int = 18;
pub(crate) const SO_SNDLOWAT: c_int = 19;
pub(crate) const SO_ACCEPTCONN: c_int = 30;
pub(crate) const SO_PEERSEC: c_int = 31;
pub(crate) const SO_SNDBUFFORCE: c_int = 32;
pub(crate) const SO_RCVBUFFORCE: c_int = 33;
pub(crate) const SO_PROTOCOL: c_int = 38;
pub(crate) const SO_DOMAIN: c_int = 39;
pub(crate) const SO_RCVTIMEO: c_int = 20;
pub(crate) const SO_SNDTIMEO: c_int = 21;
pub(crate) const SO_RCVTIMEO_NEW: c_int = 66;
pub(crate) const SO_SNDTIMEO_NEW: c_int = 67;

pub(crate) const IP_TOS: c_int = 1;
pub(crate) const IP_TTL: c_int = 2;
pub(crate) const IP_HDRINCL: c_int = 3;
pub(crate) const IP_OPTIONS: c_int = 4;
pub(crate) const IP_ROUTER_ALERT: c_int = 5;
pub(crate) const IP_RECVOPTS: c_int = 6;
pub(crate) const IP_RETOPTS: c_int = 7;
pub(crate) const IP_PKTINFO: c_int = 8;
pub(crate) const IP_PKTOPTIONS: c_int = 9;
pub(crate) const IP_MTU_DISCOVER: c_int = 10;
pub(crate) const IP_RECVERR: c_int = 11;
pub(crate) const IP_RECVTTL: c_int = 12;
pub(crate) const IP_RECVTOS: c_int = 13;
pub(crate) const IP_MTU: c_int = 14;
pub(crate) const IP_FREEBIND: c_int = 15;
pub(crate) const IP_IPSEC_POLICY: c_int = 16;
pub(crate) const IP_XFRM_POLICY: c_int = 17;
pub(crate) const IP_PASSSEC: c_int = 18;
pub(crate) const IP_TRANSPARENT: c_int = 19;
pub(crate) const IP_ORIGDSTADDR: c_int = 20;
pub(crate) const IP_MINTTL: c_int = 21;
pub(crate) const IP_NODEFRAG: c_int = 22;
pub(crate) const IP_CHECKSUM: c_int = 23;
pub(crate) const IP_BIND_ADDRESS_NO_PORT: c_int = 24;
pub(crate) const IP_RECVFRAGSIZE: c_int = 25;
pub(crate) const IP_MULTICAST_IF: c_int = 32;
pub(crate) const IP_MULTICAST_TTL: c_int = 33;
pub(crate) const IP_MULTICAST_LOOP: c_int = 34;
pub(crate) const IP_ADD_MEMBERSHIP: c_int = 35;
pub(crate) const IP_DROP_MEMBERSHIP: c_int = 36;
pub(crate) const IP_UNBLOCK_SOURCE: c_int = 37;
pub(crate) const IP_BLOCK_SOURCE: c_int = 38;
pub(crate) const IP_ADD_SOURCE_MEMBERSHIP: c_int = 39;
pub(crate) const IP_DROP_SOURCE_MEMBERSHIP: c_int = 40;
pub(crate) const IP_MSFILTER: c_int = 41;
pub(crate) const IP_MULTICAST_ALL: c_int = 49;
pub(crate) const IP_UNICAST_IF: c_int = 50;

pub(crate) const IPV6_ADDRFORM: c_int = 1;
pub(crate) const IPV6_2292PKTINFO: c_int = 2;
pub(crate) const IPV6_2292HOPOPTS: c_int = 3;
pub(crate) const IPV6_2292DSTOPTS: c_int = 4;
pub(crate) const IPV6_2292RTHDR: c_int = 5;
pub(crate) const IPV6_2292PKTOPTIONS: c_int = 6;
pub(crate) const IPV6_CHECKSUM: c_int = 7;
pub(crate) const IPV6_2292HOPLIMIT: c_int = 8;
pub(crate) const IPV6_NEXTHOP: c_int = 9;
pub(crate) const IPV6_AUTHHDR: c_int = 10;
pub(crate) const IPV6_UNICAST_HOPS: c_int = 16;
pub(crate) const IPV6_MULTICAST_IF: c_int = 17;
pub(crate) const IPV6_MULTICAST_HOPS: c_int = 18;
pub(crate) const IPV6_MULTICAST_LOOP: c_int = 19;
pub(crate) const IPV6_ADD_MEMBERSHIP: c_int = 20;
pub(crate) const IPV6_DROP_MEMBERSHIP: c_int = 21;
pub(crate) const IPV6_ROUTER_ALERT: c_int = 22;
pub(crate) const IPV6_MTU_DISCOVER: c_int = 23;
pub(crate) const IPV6_MTU: c_int = 24;
pub(crate) const IPV6_RECVERR: c_int = 25;
pub(crate) const IPV6_V6ONLY: c_int = 26;
pub(crate) const IPV6_JOIN_ANYCAST: c_int = 27;
pub(crate) const IPV6_LEAVE_ANYCAST: c_int = 28;
pub(crate) const IPV6_MULTICAST_ALL: c_int = 29;
pub(crate) const IPV6_ROUTER_ALERT_ISOLATE: c_int = 30;
pub(crate) const IPV6_IPSEC_POLICY: c_int = 34;
pub(crate) const IPV6_XFRM_POLICY: c_int = 35;
pub(crate) const IPV6_HDRINCL: c_int = 36;
pub(crate) const IPV6_RECVPKTINFO: c_int = 49;
pub(crate) const IPV6_PKTINFO: c_int = 50;
pub(crate) const IPV6_RECVHOPLIMIT: c_int = 51;
pub(crate) const IPV6_HOPLIMIT: c_int = 52;
pub(crate) const IPV6_RECVHOPOPTS: c_int = 53;
pub(crate) const IPV6_HOPOPTS: c_int = 54;
pub(crate) const IPV6_RTHDRDSTOPTS: c_int = 55;
pub(crate) const IPV6_RECVRTHDR: c_int = 56;
pub(crate) const IPV6_RTHDR: c_int = 57;
pub(crate) const IPV6_RECVDSTOPTS: c_int = 58;
pub(crate) const IPV6_DSTOPTS: c_int = 59;
pub(crate) const IPV6_RECVPATHMTU: c_int = 60;
pub(crate) const IPV6_PATHMTU: c_int = 61;
pub(crate) const IPV6_DONTFRAG: c_int = 62;
pub(crate) const IPV6_RECVTCLASS: c_int = 66;
pub(crate) const IPV6_TCLASS: c_int = 67;
pub(crate) const IPV6_AUTOFLOWLABEL: c_int = 70;
pub(crate) const IPV6_ADDR_PREFERENCES: c_int = 72;
pub(crate) const IPV6_MINHOPCOUNT: c_int = 73;
pub(crate) const IPV6_ORIGDSTADDR: c_int = 74;
pub(crate) const IPV6_TRANSPARENT: c_int = 75;
pub(crate) const IPV6_UNICAST_IF: c_int = 76;
pub(crate) const IPV6_RECVFRAGSIZE: c_int = 77;
pub(crate) const IPV6_FREEBIND: c_int = 78;

pub(crate) const TCP_NODELAY: c_int = 1;
pub(crate) const TCP_MAXSEG: c_int = 2;
pub(crate) const TCP_CORK: c_int = 3;
pub(crate) const TCP_KEEPIDLE: c_int = 4;
pub(crate) const TCP_KEEPINTVL: c_int = 5;
pub(crate) const TCP_KEEPCNT: c_int = 6;
pub(crate) const TCP_SYNCNT: c_int = 7;
pub(crate) const TCP_LINGER2: c_int = 8;
pub(crate) const TCP_DEFER_ACCEPT: c_int = 9;
pub(crate) const TCP_WINDOW_CLAMP: c_int = 10;
pub(crate) const TCP_INFO: c_int = 11;
pub(crate) const TCP_QUICKACK: c_int = 12;
pub(crate) const TCP_CONGESTION: c_int = 13;
pub(crate) const TCP_MD5SIG: c_int = 14;
pub(crate) const TCP_THIN_LINEAR_TIMEOUTS: c_int = 16;
pub(crate) const TCP_THIN_DUPACK: c_int = 17;
pub(crate) const TCP_USER_TIMEOUT: c_int = 18;
pub(crate) const TCP_REPAIR: c_int = 19;
pub(crate) const TCP_REPAIR_QUEUE: c_int = 20;
pub(crate) const TCP_QUEUE_SEQ: c_int = 21;
pub(crate) const TCP_REPAIR_OPTIONS: c_int = 22;
pub(crate) const TCP_FASTOPEN: c_int = 23;
pub(crate) const TCP_TIMESTAMP: c_int = 24;
pub(crate) const TCP_NOTSENT_LOWAT: c_int = 25;
pub(crate) const TCP_CC_INFO: c_int = 26;
pub(crate) const TCP_SAVE_SYN: c_int = 27;
pub(crate) const TCP_SAVED_SYN: c_int = 28;
pub(crate) const TCP_REPAIR_WINDOW: c_int = 29;
pub(crate) const TCP_FASTOPEN_CONNECT: c_int = 30;
pub(crate) const TCP_ULP: c_int = 31;
pub(crate) const TCP_MD5SIG_EXT: c_int = 32;
pub(crate) const TCP_FASTOPEN_KEY: c_int = 33;
pub(crate) const TCP_FASTOPEN_NO_COOKIE: c_int = 34;
pub(crate) const TCP_ZEROCOPY_RECEIVE: c_int = 35;
pub(crate) const TCP_INQ: c_int = 36;

pub(crate) const EAI_NONAME: c_int = -2;
pub(crate) const EAI_SYSTEM: c_int = -11;

pub(crate) const FUTEX_WAIT: c_int = 0;
pub(crate) const FUTEX_WAKE: c_int = 1;
pub(crate) const FUTEX_FD: c_int = 2;
pub(crate) const FUTEX_REQUEUE: c_int = 3;
pub(crate) const FUTEX_CMP_REQUEUE: c_int = 4;
pub(crate) const FUTEX_WAKE_OP: c_int = 5;
pub(crate) const FUTEX_LOCK_PI: c_int = 6;
pub(crate) const FUTEX_UNLOCK_PI: c_int = 7;
pub(crate) const FUTEX_TRYLOCK_PI: c_int = 8;
pub(crate) const FUTEX_WAIT_BITSET: c_int = 9;

pub(crate) const PTHREAD_MUTEX_NORMAL: c_int = 0;
pub(crate) const PTHREAD_MUTEX_DEFAULT: c_int = 0;
pub(crate) const PTHREAD_MUTEX_RECURSIVE: c_int = 1;
pub(crate) const PTHREAD_MUTEX_ERRORCHECK: c_int = 2;

pub(crate) const PR_SET_NAME: c_int = 15;

#[repr(C)]
pub(crate) struct Dirent64 {
    pub(crate) d_ino: u64,
    pub(crate) d_off: u64,
    pub(crate) d_reclen: u16,
    pub(crate) d_type: u8,
    pub(crate) d_name: [u8; 256],
}

pub(crate) type SockLen = u32;

#[repr(C)]
pub(crate) struct Addrinfo {
    pub(crate) ai_flags: c_int,
    pub(crate) ai_family: c_int,
    pub(crate) ai_socktype: c_int,
    pub(crate) ai_protocol: c_int,
    pub(crate) ai_addrlen: SockLen,
    pub(crate) ai_addr: *mut rustix::net::SocketAddrStorage,
    pub(crate) ai_canonname: *mut i8,
    pub(crate) ai_next: *mut Addrinfo,
}

#[repr(C)]
pub(crate) struct Linger {
    pub(crate) l_onoff: c_int,
    pub(crate) l_linger: c_int,
}

#[repr(C)]
pub(crate) struct Ipv4Addr {
    pub(crate) s_addr: u32,
}

#[repr(C)]
#[repr(align(4))]
pub(crate) struct Ipv6Addr {
    pub(crate) u6_addr8: [u8; 16_usize],
}

#[repr(C)]
pub(crate) struct Ipv4Mreq {
    pub(crate) imr_multiaddr: Ipv4Addr,
    pub(crate) imr_interface: Ipv4Addr,
}

#[repr(C)]
pub(crate) struct Ipv6Mreq {
    pub(crate) ipv6mr_multiaddr: Ipv6Addr,
    pub(crate) ipv6mr_interface: c_int,
}

#[repr(C)]
#[cfg_attr(
    any(
        all(
            target_arch = "x86",
            not(target_env = "musl"),
            not(target_os = "android")
        ),
        target_arch = "x86_64"
    ),
    repr(packed)
)]
pub struct EpollEvent {
    pub events: u32,
    pub u64: u64,
}

#[repr(C)]
pub(crate) struct OldTimespec {
    pub(crate) tv_sec: OldTime,
    pub(crate) tv_nsec: Suseconds,
}

#[repr(C)]
pub(crate) struct OldTimeval {
    pub(crate) tv_sec: OldTime,
    pub(crate) tv_usec: Suseconds,
}

pub(crate) type OldTime = c_long;
pub(crate) type Suseconds = c_long;

pub(crate) const AT_NULL: c_ulong = 0;
pub(crate) const AT_IGNORE: c_ulong = 1;
pub(crate) const AT_EXECFD: c_ulong = 2;
pub(crate) const AT_PHDR: c_ulong = 3;
pub(crate) const AT_PHENT: c_ulong = 4;
pub(crate) const AT_PHNUM: c_ulong = 5;
pub(crate) const AT_PAGESZ: c_ulong = 6;
pub(crate) const AT_BASE: c_ulong = 7;
pub(crate) const AT_FLAGS: c_ulong = 8;
pub(crate) const AT_ENTRY: c_ulong = 9;
pub(crate) const AT_NOTELF: c_ulong = 10;
pub(crate) const AT_UID: c_ulong = 11;
pub(crate) const AT_EUID: c_ulong = 12;
pub(crate) const AT_GID: c_ulong = 13;
pub(crate) const AT_EGID: c_ulong = 14;
pub(crate) const AT_CLKTCK: c_ulong = 17;
pub(crate) const AT_PLATFORM: c_ulong = 15;
pub(crate) const AT_HWCAP: c_ulong = 16;
pub(crate) const AT_FPUCW: c_ulong = 18;
pub(crate) const AT_DCACHEBSIZE: c_ulong = 19;
pub(crate) const AT_ICACHEBSIZE: c_ulong = 20;
pub(crate) const AT_UCACHEBSIZE: c_ulong = 21;
pub(crate) const AT_IGNOREPPC: c_ulong = 22;
pub(crate) const AT_BASE_PLATFORM: c_ulong = 24;
pub(crate) const AT_RANDOM: c_ulong = 25;
pub(crate) const AT_HWCAP2: c_ulong = 26;
pub(crate) const AT_EXECFN: c_ulong = 31;
pub(crate) const AT_SYSINFO: c_ulong = 32;
pub(crate) const AT_SYSINFO_EHDR: c_ulong = 33;

#[repr(C)]
pub(crate) struct DlPhdrInfo {
    pub(crate) dlpi_addr: usize,
    pub(crate) dlpi_name: *const c_char,
    pub(crate) dlpi_phdr: *const Elf_Phdr,
    pub(crate) dlpi_phnum: u16,
    // We don't yet implement these, so leave them out. Users should be
    // checking the size passed to the `dl_iterate_phdr` callback before
    // checking these fields.
    /*
    pub(crate) dlpi_adds: c_ulonglong,
    pub(crate) dlpi_subs: c_ulonglong,
    pub(crate) dlpi_tls_modid: usize,
    pub(crate) dlpi_tls_data: *mut c_void,
    */
}

#[cfg(target_pointer_width = "32")]
#[repr(C)]
pub(crate) struct Elf_Phdr {
    pub(crate) p_type: u32,
    pub(crate) p_offset: u32,
    pub(crate) p_vaddr: u32,
    pub(crate) p_paddr: u32,
    pub(crate) p_filesz: u32,
    pub(crate) p_memsz: u32,
    pub(crate) p_flags: u32,
    pub(crate) p_align: u32,
}

#[cfg(target_pointer_width = "64")]
#[repr(C)]
pub(crate) struct Elf_Phdr {
    pub(crate) p_type: u32,
    pub(crate) p_flags: u32,
    pub(crate) p_offset: u64,
    pub(crate) p_vaddr: u64,
    pub(crate) p_paddr: u64,
    pub(crate) p_filesz: u64,
    pub(crate) p_memsz: u64,
    pub(crate) p_align: u64,
}
