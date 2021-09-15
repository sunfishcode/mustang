#![allow(non_upper_case_globals)]
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
constant_same_as!(
    ALIGNOF_MAXALIGN_T,
    std::mem::align_of::<libc::max_align_t>()
);
constant!(MAP_ANONYMOUS);
constant_same_as!(MAP_FAILED, libc::MAP_FAILED);
constant!(_SC_PAGESIZE);
constant!(_SC_GETPW_R_SIZE_MAX);
constant!(_SC_NPROCESSORS_ONLN);
constant!(_SC_SYMLOOP_MAX);
constant_same_as!(SYMLOOP_MAX, unsafe { libc::sysconf(libc::_SC_SYMLOOP_MAX) });
constant!(SYS_getrandom);
constant!(CLOCK_MONOTONIC);
constant!(CLOCK_REALTIME);
constant!(SIG_DFL);
constant!(SHUT_RD);
constant!(SHUT_WR);
constant!(SHUT_RDWR);
constant!(AF_UNIX);
constant!(AF_INET);
constant!(AF_INET6);
constant!(AF_NETLINK);
constant!(SOCK_STREAM);
constant!(SOCK_DGRAM);
constant!(SOCK_RAW);
constant!(SOCK_RDM);
constant!(SOCK_SEQPACKET);
constant!(IPPROTO_IP);
constant!(IPPROTO_ICMP);
constant!(IPPROTO_IGMP);
constant!(IPPROTO_IPIP);
constant!(IPPROTO_TCP);
constant!(IPPROTO_EGP);
constant!(IPPROTO_PUP);
constant!(IPPROTO_UDP);
constant!(IPPROTO_IDP);
constant!(IPPROTO_TP);
constant!(IPPROTO_DCCP);
constant!(IPPROTO_IPV6);
constant!(IPPROTO_RSVP);
constant!(IPPROTO_GRE);
constant!(IPPROTO_ESP);
constant!(IPPROTO_AH);
constant!(IPPROTO_MTP);
constant!(IPPROTO_BEETPH);
constant!(IPPROTO_ENCAP);
constant!(IPPROTO_PIM);
constant!(IPPROTO_COMP);
constant!(IPPROTO_SCTP);
constant!(IPPROTO_UDPLITE);
constant!(IPPROTO_MPLS);
constant!(IPPROTO_ETHERNET);
constant!(IPPROTO_RAW);
constant!(IPPROTO_MPTCP);
type_!(Dirent64, dirent64);
type_!(SockLen, socklen_t);
/* FIXME: implement getsockopt, setsockopt, getaddrinfo, and freeaddrinfo
 *type_!(Addrinfo, addrinfo);
 */
