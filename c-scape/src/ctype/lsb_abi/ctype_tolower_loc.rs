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
    50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 'a' as i32, 'b' as i32, 'c' as i32,
    'd' as i32, 'e' as i32, 'f' as i32, 'g' as i32, 'h' as i32, 'i' as i32, 'j' as i32, 'k' as i32,
    'l' as i32, 'm' as i32, 'n' as i32, 'o' as i32, 'p' as i32, 'q' as i32, 'r' as i32, 's' as i32,
    't' as i32, 'u' as i32, 'v' as i32, 'w' as i32, 'x' as i32, 'y' as i32, 'z' as i32, 91, 92, 93,
    94, 95, 96, 'a' as i32, 'b' as i32, 'c' as i32, 'd' as i32, 'e' as i32, 'f' as i32, 'g' as i32,
    'h' as i32, 'i' as i32, 'j' as i32, 'k' as i32, 'l' as i32, 'm' as i32, 'n' as i32, 'o' as i32,
    'p' as i32, 'q' as i32, 'r' as i32, 's' as i32, 't' as i32, 'u' as i32, 'v' as i32, 'w' as i32,
    'x' as i32, 'y' as i32, 'z' as i32, 123, 124, 125, 126, 127, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

static PTR: SyncUnsafeCell<SyncConstPtr<i32>> =
    SyncUnsafeCell::new(unsafe { SyncConstPtr::new(addr_of!(TABLE[128])) });

// https://refspecs.linuxbase.org/LSB_5.0.0/LSB-Core-generic/LSB-Core-generic/libutil---ctype-tolower-loc.html
#[no_mangle]
extern "C" fn __ctype_tolower_loc() -> *mut *const i32 {
    SyncUnsafeCell::get(&PTR) as *mut _ as *mut *const i32
}
