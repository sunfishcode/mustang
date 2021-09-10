// Constants and types used in `*-*-linux-gnu` targets.

use std::os::raw::{c_int, c_long};
use std::ffi::c_void;

pub const F_SETFD: c_int = 2;
pub const F_GETFL: c_int = 3;
pub const F_DUPFD_CLOEXEC: c_int = 1030;

pub const SEEK_SET: c_int = 0;
pub const SEEK_CUR: c_int = 1;
pub const SEEK_END: c_int = 2;

pub const DT_UNKNOWN: u8 = 0;
pub const DT_FIFO: u8 = 1;
pub const DT_CHR: u8 = 2;
pub const DT_DIR: u8 = 4;
pub const DT_BLK: u8 = 6;
pub const DT_REG: u8 = 8;
pub const DT_LNK: u8 = 10;
pub const DT_SOCK: u8 = 12;

pub const TCGETS: c_long = 0x5401;

pub const SIZEOF_MAXALIGN_T: usize = 32;

pub const MAP_ANONYMOUS: i32 = 32;

pub const MAP_FAILED: *mut c_void = -1_isize as usize as *mut c_void;

pub const _SC_PAGESIZE: c_int = 30;
pub const _SC_GETPW_R_SIZE_MAX: c_int = 70;
pub const _SC_NPROCESSORS_ONLN: c_int = 84;
pub const _SC_SYMLOOP_MAX: c_int = 173;

pub const SYMLOOP_MAX: c_long = 40;

pub const SYS_getrandom: c_long = 318;

pub const CLOCK_REALTIME: c_int = 0;
pub const CLOCK_MONOTONIC: c_int = 1;
