#![allow(non_upper_case_globals)]
#![allow(unused_imports)]
#![cfg_attr(test, allow(non_snake_case))]

#[cfg(all(target_os = "linux", target_env = "gnu"))]
mod linux_gnu;

#[cfg(all(target_os = "linux", target_env = "gnu"))]
use linux_gnu as cfg;

// Macros to export constants and check that they match the values in libc.

macro_rules! constant {
    ($name:ident) => {
        pub(crate) use cfg::$name;
        #[cfg(test)]
        static_assertions::const_assert_eq!($name, libc::$name as _);
        #[cfg(test)]
        static_assertions::const_assert_eq!(libc::$name, $name as _);
    };
}

macro_rules! constant_same_as {
    ($name:ident, $value:expr) => {
        pub(crate) use cfg::$name;
        #[cfg(test)]
        paste::paste! {
            #[test]
            fn [< test_ $name >]() {
                assert_eq!($name, $value as _);
                assert_eq!($value, $name as _);
            }
        }
    };
}

macro_rules! type_ {
    ($name:ident, $libc:ident) => {
        pub(crate) use cfg::$name;

        #[cfg(test)]
        static_assertions::const_assert_eq!(
            std::mem::size_of::<$name>(),
            std::mem::size_of::<libc::$libc>()
        );
        #[cfg(test)]
        static_assertions::const_assert_eq!(
            std::mem::align_of::<$name>(),
            std::mem::align_of::<libc::$libc>()
        );
    };
}

macro_rules! struct_ {
    ($name:ident, $libc:ident, $($field:ident),*) => {
        // Check the size and alignment.
        type_!($name, $libc);

        // Check that the fields match libc's fields.
        #[cfg(test)]
        paste::paste! {
            #[allow(dead_code)]
            const [< TEST_ $name >]: once_cell::sync::Lazy<()> = once_cell::sync::Lazy::new(|| {
                #[allow(non_snake_case)]
                let [< struct_ $name >]: $name = $name {
                    $($field: 0 as _),*
                };
                #[allow(non_snake_case)]
                let [< _libc_ $name >]: libc::$libc = libc::$libc {
                    $($field: [< struct_ $name >].$field),*
                };
            });
        }
    };
}

constant!(F_SETFD);
constant!(F_GETFL);
constant!(F_DUPFD_CLOEXEC);
constant!(SEEK_SET);
constant!(SEEK_CUR);
constant!(SEEK_END);
constant!(DT_UNKNOWN);
constant!(DT_FIFO);
constant!(DT_CHR);
constant!(DT_DIR);
constant!(DT_BLK);
constant!(DT_REG);
constant!(DT_LNK);
constant!(DT_SOCK);
constant!(TCGETS);
constant!(FIONBIO);
#[cfg(not(target_arch = "riscv64"))] // libc for riscv64 doesn't have max_align_t yet
constant_same_as!(
    ALIGNOF_MAXALIGN_T,
    std::mem::align_of::<libc::max_align_t>()
);
#[cfg(target_arch = "riscv64")]
constant_same_as!(ALIGNOF_MAXALIGN_T, 16);
constant!(MAP_ANONYMOUS);
constant!(MREMAP_MAYMOVE);
constant!(MREMAP_FIXED);
constant!(MREMAP_DONTUNMAP);
constant_same_as!(MAP_FAILED, libc::MAP_FAILED);
constant!(_SC_PAGESIZE);
constant!(_SC_GETPW_R_SIZE_MAX);
constant!(_SC_NPROCESSORS_ONLN);
constant!(_SC_SYMLOOP_MAX);
constant_same_as!(SYMLOOP_MAX, unsafe { libc::sysconf(libc::_SC_SYMLOOP_MAX) });
constant!(SYS_getrandom);
constant!(SYS_futex);
constant!(SYS_clone3);
constant!(CLOCK_MONOTONIC);
constant!(CLOCK_REALTIME);
constant!(SIG_DFL);
constant!(SHUT_RD);
constant!(SHUT_WR);
constant!(SHUT_RDWR);
constant!(SOL_SOCKET);
constant!(IPPROTO_IP);
constant!(IPPROTO_IPV6);
constant!(IPPROTO_TCP);
constant!(SO_REUSEADDR);
constant!(SO_BROADCAST);
constant!(SO_PASSCRED);
constant!(SO_DEBUG);
constant!(SO_TYPE);
constant!(SO_ERROR);
constant!(SO_DONTROUTE);
constant!(SO_SNDBUF);
constant!(SO_RCVBUF);
constant!(SO_KEEPALIVE);
constant!(SO_OOBINLINE);
constant!(SO_NO_CHECK);
constant!(SO_PRIORITY);
constant!(SO_LINGER);
constant!(SO_BSDCOMPAT);
constant!(SO_REUSEPORT);
constant!(SO_RCVLOWAT);
constant!(SO_SNDLOWAT);
constant!(SO_PEERCRED);
constant!(SO_ACCEPTCONN);
constant!(SO_PEERSEC);
constant!(SO_SNDBUFFORCE);
constant!(SO_RCVBUFFORCE);
constant!(SO_PROTOCOL);
constant!(SO_DOMAIN);
constant!(SO_SNDTIMEO);
constant!(SO_RCVTIMEO);
constant!(IP_TOS);
constant!(IP_TTL);
constant!(IP_HDRINCL);
constant!(IP_OPTIONS);
constant!(IP_ROUTER_ALERT);
constant!(IP_RECVOPTS);
constant!(IP_RETOPTS);
constant!(IP_PKTINFO);
constant!(IP_PKTOPTIONS);
constant!(IP_MTU_DISCOVER);
constant!(IP_RECVERR);
constant!(IP_RECVTTL);
constant!(IP_RECVTOS);
constant!(IP_MTU);
constant!(IP_FREEBIND);
constant!(IP_IPSEC_POLICY);
constant!(IP_XFRM_POLICY);
constant!(IP_PASSSEC);
constant!(IP_TRANSPARENT);
constant!(IP_ORIGDSTADDR);
constant!(IP_MINTTL);
constant!(IP_NODEFRAG);
constant!(IP_CHECKSUM);
constant!(IP_BIND_ADDRESS_NO_PORT);
constant!(IP_RECVFRAGSIZE);
constant!(IP_MULTICAST_IF);
constant!(IP_MULTICAST_TTL);
constant!(IP_MULTICAST_LOOP);
constant!(IP_ADD_MEMBERSHIP);
constant!(IP_DROP_MEMBERSHIP);
constant!(IP_UNBLOCK_SOURCE);
constant!(IP_BLOCK_SOURCE);
constant!(IP_ADD_SOURCE_MEMBERSHIP);
constant!(IP_DROP_SOURCE_MEMBERSHIP);
constant!(IP_MSFILTER);
constant!(IP_MULTICAST_ALL);
constant!(IP_UNICAST_IF);
constant!(IPV6_ADDRFORM);
constant!(IPV6_2292PKTINFO);
constant!(IPV6_2292HOPOPTS);
constant!(IPV6_2292DSTOPTS);
constant!(IPV6_2292RTHDR);
constant!(IPV6_2292PKTOPTIONS);
constant!(IPV6_CHECKSUM);
constant!(IPV6_2292HOPLIMIT);
constant!(IPV6_NEXTHOP);
constant!(IPV6_AUTHHDR);
constant!(IPV6_UNICAST_HOPS);
constant!(IPV6_MULTICAST_IF);
constant!(IPV6_MULTICAST_HOPS);
constant!(IPV6_MULTICAST_LOOP);
constant!(IPV6_ADD_MEMBERSHIP);
constant!(IPV6_DROP_MEMBERSHIP);
constant!(IPV6_ROUTER_ALERT);
constant!(IPV6_MTU_DISCOVER);
constant!(IPV6_MTU);
constant!(IPV6_RECVERR);
constant!(IPV6_V6ONLY);
constant!(IPV6_JOIN_ANYCAST);
constant!(IPV6_LEAVE_ANYCAST);
constant!(IPV6_MULTICAST_ALL);
constant!(IPV6_ROUTER_ALERT_ISOLATE);
constant!(IPV6_IPSEC_POLICY);
constant!(IPV6_XFRM_POLICY);
constant!(IPV6_HDRINCL);
constant!(IPV6_RECVPKTINFO);
constant!(IPV6_PKTINFO);
constant!(IPV6_RECVHOPLIMIT);
constant!(IPV6_HOPLIMIT);
constant!(IPV6_RECVHOPOPTS);
constant!(IPV6_HOPOPTS);
constant!(IPV6_RTHDRDSTOPTS);
constant!(IPV6_RECVRTHDR);
constant!(IPV6_RTHDR);
constant!(IPV6_RECVDSTOPTS);
constant!(IPV6_DSTOPTS);
constant!(IPV6_RECVPATHMTU);
constant!(IPV6_PATHMTU);
constant!(IPV6_DONTFRAG);
constant!(IPV6_RECVTCLASS);
constant!(IPV6_TCLASS);
constant!(IPV6_AUTOFLOWLABEL);
constant!(IPV6_ADDR_PREFERENCES);
constant!(IPV6_MINHOPCOUNT);
constant!(IPV6_ORIGDSTADDR);
constant!(IPV6_TRANSPARENT);
constant!(IPV6_UNICAST_IF);
constant!(IPV6_RECVFRAGSIZE);
constant!(IPV6_FREEBIND);
constant!(TCP_NODELAY);
constant!(TCP_MAXSEG);
constant!(TCP_CORK);
constant!(TCP_KEEPIDLE);
constant!(TCP_KEEPINTVL);
constant!(TCP_KEEPCNT);
constant!(TCP_SYNCNT);
constant!(TCP_LINGER2);
constant!(TCP_DEFER_ACCEPT);
constant!(TCP_WINDOW_CLAMP);
constant!(TCP_INFO);
constant!(TCP_QUICKACK);
constant!(TCP_CONGESTION);
constant!(TCP_MD5SIG);
constant!(TCP_THIN_LINEAR_TIMEOUTS);
constant!(TCP_THIN_DUPACK);
constant!(TCP_USER_TIMEOUT);
constant!(TCP_REPAIR);
constant!(TCP_REPAIR_QUEUE);
constant!(TCP_QUEUE_SEQ);
constant!(TCP_REPAIR_OPTIONS);
constant!(TCP_FASTOPEN);
constant!(TCP_TIMESTAMP);
constant!(TCP_NOTSENT_LOWAT);
constant!(TCP_CC_INFO);
constant!(TCP_SAVE_SYN);
constant!(TCP_SAVED_SYN);
constant!(TCP_REPAIR_WINDOW);
constant!(TCP_FASTOPEN_CONNECT);
constant!(TCP_ULP);
constant!(TCP_MD5SIG_EXT);
constant!(TCP_FASTOPEN_KEY);
constant!(TCP_FASTOPEN_NO_COOKIE);
constant!(TCP_ZEROCOPY_RECEIVE);
constant!(TCP_INQ);
constant!(EAI_NONAME);
constant!(EAI_SYSTEM);
constant!(FUTEX_WAIT);
constant!(FUTEX_WAKE);
constant!(FUTEX_FD);
constant!(FUTEX_REQUEUE);
constant!(FUTEX_CMP_REQUEUE);
constant!(FUTEX_WAKE_OP);
constant!(FUTEX_LOCK_PI);
constant!(FUTEX_UNLOCK_PI);
constant!(FUTEX_TRYLOCK_PI);
constant!(FUTEX_WAIT_BITSET);
constant!(PTHREAD_MUTEX_NORMAL);
constant!(PTHREAD_MUTEX_DEFAULT);
constant!(PTHREAD_MUTEX_RECURSIVE);
constant!(PTHREAD_MUTEX_ERRORCHECK);
constant!(PR_SET_NAME);
constant!(AT_NULL);
constant!(AT_IGNORE);
constant!(AT_EXECFD);
constant!(AT_PHDR);
constant!(AT_PHENT);
constant!(AT_PHNUM);
constant!(AT_PAGESZ);
constant!(AT_BASE);
constant!(AT_FLAGS);
constant!(AT_ENTRY);
constant!(AT_NOTELF);
constant!(AT_UID);
constant!(AT_EUID);
constant!(AT_GID);
constant!(AT_EGID);
constant!(AT_CLKTCK);
constant!(AT_PLATFORM);
constant!(AT_HWCAP);
constant!(AT_FPUCW);
constant!(AT_DCACHEBSIZE);
constant!(AT_ICACHEBSIZE);
constant!(AT_UCACHEBSIZE);
constant!(AT_IGNOREPPC);
constant!(AT_BASE_PLATFORM);
constant!(AT_RANDOM);
constant!(AT_HWCAP2);
constant!(AT_EXECFN);
constant!(AT_SYSINFO);
constant!(AT_SYSINFO_EHDR);
type_!(Dirent64, dirent64);
type_!(SockLen, socklen_t);
type_!(Addrinfo, addrinfo);
type_!(Linger, linger);
type_!(Ipv4Addr, in_addr);
type_!(Ipv6Addr, in6_addr);
type_!(Ipv4Mreq, ip_mreq);
type_!(Ipv6Mreq, ipv6_mreq);
struct_!(EpollEvent, epoll_event, events, u64);
struct_!(OldTimeval, timeval, tv_sec, tv_usec);
struct_!(OldTimespec, timespec, tv_sec, tv_nsec);
struct_!(CpuSet, cpu_set_t, bits);
type_!(OldTime, time_t);
type_!(Suseconds, suseconds_t);
// This differs from libc, as we don't yet implement the optional fields
// at the end.
/*
struct_!(
    DlPhdrInfo,
    dl_phdr_info,
    dlpi_addr,
    dlpi_name,
    dlpi_phdr,
    dlpi_phnum,
);
*/
pub(crate) use cfg::DlPhdrInfo;
#[cfg(target_pointer_width = "32")]
struct_!(
    Elf_Phdr, Elf32_Phdr, p_type, p_offset, p_vaddr, p_paddr, p_filesz, p_memsz, p_flags, p_align
);
#[cfg(target_pointer_width = "64")]
struct_!(
    Elf_Phdr, Elf64_Phdr, p_type, p_flags, p_offset, p_vaddr, p_paddr, p_filesz, p_memsz, p_align
);
