use core::cell::SyncUnsafeCell;
use core::ptr::addr_of;

use crate::sync_ptr::SyncConstPtr;

static TABLE: [i32; 128 + 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49,
    50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 'A' as i32, 'B' as i32, 'C' as i32,
    'D' as i32, 'E' as i32, 'F' as i32, 'G' as i32, 'H' as i32, 'I' as i32, 'J' as i32, 'K' as i32,
    'L' as i32, 'M' as i32, 'N' as i32, 'O' as i32, 'P' as i32, 'Q' as i32, 'R' as i32, 'S' as i32,
    'T' as i32, 'U' as i32, 'V' as i32, 'W' as i32, 'X' as i32, 'Y' as i32, 'Z' as i32, 91, 92, 93,
    94, 95, 96, 'A' as i32, 'B' as i32, 'C' as i32, 'D' as i32, 'E' as i32, 'F' as i32, 'G' as i32,
    'H' as i32, 'I' as i32, 'J' as i32, 'K' as i32, 'L' as i32, 'M' as i32, 'N' as i32, 'O' as i32,
    'P' as i32, 'Q' as i32, 'R' as i32, 'S' as i32, 'T' as i32, 'U' as i32, 'V' as i32, 'W' as i32,
    'X' as i32, 'Y' as i32, 'Z' as i32, 123, 124, 125, 126, 127, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

static PTR: SyncUnsafeCell<SyncConstPtr<i32>> =
    SyncUnsafeCell::new(unsafe { SyncConstPtr::new(addr_of!(TABLE[128])) });

// https://refspecs.linuxbase.org/LSB_5.0.0/LSB-Core-generic/LSB-Core-generic/libutil---ctype-toupper-loc.html
#[no_mangle]
extern "C" fn __ctype_toupper_loc() -> *mut *const i32 {
    SyncUnsafeCell::get(&PTR) as *mut _ as *mut *const i32
}
