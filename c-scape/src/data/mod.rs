#![allow(non_upper_case_globals)]
#![cfg_attr(test, allow(non_snake_case))]

#[cfg(all(target_os = "linux", target_env = "gnu"))]
mod linux_gnu;

#[cfg(all(target_os = "linux", target_env = "gnu"))]
use linux_gnu as cfg;

macro_rules! constant {
    ($name:ident) => {
        pub(crate) use cfg::$name;
        #[cfg(test)]
        static_assertions::const_assert_eq!($name, libc::$name as _);
        #[cfg(test)]
        static_assertions::const_assert_eq!(libc::$name, $name as _);
    };
}

macro_rules! dynamic {
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
dynamic!(SIZEOF_MAXALIGN_T, std::mem::size_of::<libc::max_align_t>());
constant!(MAP_ANONYMOUS);
dynamic!(MAP_FAILED, libc::MAP_FAILED);
constant!(_SC_PAGESIZE);
constant!(_SC_GETPW_R_SIZE_MAX);
constant!(_SC_NPROCESSORS_ONLN);
constant!(_SC_SYMLOOP_MAX);
dynamic!(SYMLOOP_MAX, unsafe { libc::sysconf(libc::_SC_SYMLOOP_MAX) });
constant!(SYS_getrandom);
constant!(CLOCK_MONOTONIC);
constant!(CLOCK_REALTIME);
