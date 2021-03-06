use core::cell::SyncUnsafeCell;
use core::ptr::addr_of;

use libc::c_ushort;

use crate::sync_ptr::SyncConstPtr;

static TABLE: [c_ushort; 128 + 256] = [
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    c_ushort::from_be(0x200),
    c_ushort::from_be(0x200),
    c_ushort::from_be(0x200),
    c_ushort::from_be(0x200),
    c_ushort::from_be(0x200),
    c_ushort::from_be(0x200),
    c_ushort::from_be(0x200),
    c_ushort::from_be(0x200),
    c_ushort::from_be(0x200),
    c_ushort::from_be(0x320),
    c_ushort::from_be(0x220),
    c_ushort::from_be(0x220),
    c_ushort::from_be(0x220),
    c_ushort::from_be(0x220),
    c_ushort::from_be(0x200),
    c_ushort::from_be(0x200),
    c_ushort::from_be(0x200),
    c_ushort::from_be(0x200),
    c_ushort::from_be(0x200),
    c_ushort::from_be(0x200),
    c_ushort::from_be(0x200),
    c_ushort::from_be(0x200),
    c_ushort::from_be(0x200),
    c_ushort::from_be(0x200),
    c_ushort::from_be(0x200),
    c_ushort::from_be(0x200),
    c_ushort::from_be(0x200),
    c_ushort::from_be(0x200),
    c_ushort::from_be(0x200),
    c_ushort::from_be(0x200),
    c_ushort::from_be(0x200),
    c_ushort::from_be(0x200),
    c_ushort::from_be(0x160),
    c_ushort::from_be(0x4c0),
    c_ushort::from_be(0x4c0),
    c_ushort::from_be(0x4c0),
    c_ushort::from_be(0x4c0),
    c_ushort::from_be(0x4c0),
    c_ushort::from_be(0x4c0),
    c_ushort::from_be(0x4c0),
    c_ushort::from_be(0x4c0),
    c_ushort::from_be(0x4c0),
    c_ushort::from_be(0x4c0),
    c_ushort::from_be(0x4c0),
    c_ushort::from_be(0x4c0),
    c_ushort::from_be(0x4c0),
    c_ushort::from_be(0x4c0),
    c_ushort::from_be(0x4c0),
    c_ushort::from_be(0x8d8),
    c_ushort::from_be(0x8d8),
    c_ushort::from_be(0x8d8),
    c_ushort::from_be(0x8d8),
    c_ushort::from_be(0x8d8),
    c_ushort::from_be(0x8d8),
    c_ushort::from_be(0x8d8),
    c_ushort::from_be(0x8d8),
    c_ushort::from_be(0x8d8),
    c_ushort::from_be(0x8d8),
    c_ushort::from_be(0x4c0),
    c_ushort::from_be(0x4c0),
    c_ushort::from_be(0x4c0),
    c_ushort::from_be(0x4c0),
    c_ushort::from_be(0x4c0),
    c_ushort::from_be(0x4c0),
    c_ushort::from_be(0x4c0),
    c_ushort::from_be(0x8d5),
    c_ushort::from_be(0x8d5),
    c_ushort::from_be(0x8d5),
    c_ushort::from_be(0x8d5),
    c_ushort::from_be(0x8d5),
    c_ushort::from_be(0x8d5),
    c_ushort::from_be(0x8c5),
    c_ushort::from_be(0x8c5),
    c_ushort::from_be(0x8c5),
    c_ushort::from_be(0x8c5),
    c_ushort::from_be(0x8c5),
    c_ushort::from_be(0x8c5),
    c_ushort::from_be(0x8c5),
    c_ushort::from_be(0x8c5),
    c_ushort::from_be(0x8c5),
    c_ushort::from_be(0x8c5),
    c_ushort::from_be(0x8c5),
    c_ushort::from_be(0x8c5),
    c_ushort::from_be(0x8c5),
    c_ushort::from_be(0x8c5),
    c_ushort::from_be(0x8c5),
    c_ushort::from_be(0x8c5),
    c_ushort::from_be(0x8c5),
    c_ushort::from_be(0x8c5),
    c_ushort::from_be(0x8c5),
    c_ushort::from_be(0x8c5),
    c_ushort::from_be(0x4c0),
    c_ushort::from_be(0x4c0),
    c_ushort::from_be(0x4c0),
    c_ushort::from_be(0x4c0),
    c_ushort::from_be(0x4c0),
    c_ushort::from_be(0x4c0),
    c_ushort::from_be(0x8d6),
    c_ushort::from_be(0x8d6),
    c_ushort::from_be(0x8d6),
    c_ushort::from_be(0x8d6),
    c_ushort::from_be(0x8d6),
    c_ushort::from_be(0x8d6),
    c_ushort::from_be(0x8c6),
    c_ushort::from_be(0x8c6),
    c_ushort::from_be(0x8c6),
    c_ushort::from_be(0x8c6),
    c_ushort::from_be(0x8c6),
    c_ushort::from_be(0x8c6),
    c_ushort::from_be(0x8c6),
    c_ushort::from_be(0x8c6),
    c_ushort::from_be(0x8c6),
    c_ushort::from_be(0x8c6),
    c_ushort::from_be(0x8c6),
    c_ushort::from_be(0x8c6),
    c_ushort::from_be(0x8c6),
    c_ushort::from_be(0x8c6),
    c_ushort::from_be(0x8c6),
    c_ushort::from_be(0x8c6),
    c_ushort::from_be(0x8c6),
    c_ushort::from_be(0x8c6),
    c_ushort::from_be(0x8c6),
    c_ushort::from_be(0x8c6),
    c_ushort::from_be(0x4c0),
    c_ushort::from_be(0x4c0),
    c_ushort::from_be(0x4c0),
    c_ushort::from_be(0x4c0),
    c_ushort::from_be(0x200),
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
];

static PTR: SyncUnsafeCell<SyncConstPtr<c_ushort>> =
    SyncUnsafeCell::new(unsafe { SyncConstPtr::new(addr_of!(TABLE[128])) });

// https://refspecs.linuxbase.org/LSB_5.0.0/LSB-Core-generic/LSB-Core-generic/baselib---ctype-b-loc.html
#[no_mangle]
extern "C" fn __ctype_b_loc() -> *mut *const c_ushort {
    SyncUnsafeCell::get(&PTR) as *mut _ as *mut *const c_ushort
}
