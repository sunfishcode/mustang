use libc::{c_int, c_uint};

use core::cell::SyncUnsafeCell;

static STATE: SyncUnsafeCell<c_uint> = SyncUnsafeCell::new(1);

// The algorithim used for this is Mulberry32, which
// is under the CC0 license.
// https://gist.github.com/tommyettinger/46a874533244883189143505d203312c

#[cfg(test)]
static_assertions::assert_eq_size!(c_uint, u32);

#[no_mangle]
unsafe extern "C" fn rand() -> c_int {
    libc!(libc::rand());

    rand_r(STATE.get())
}

#[no_mangle]
unsafe extern "C" fn srand(seed: c_uint) {
    libc!(libc::srand(seed));

    *STATE.get() = seed;
}

// marked as obsolete for some reason
#[no_mangle]
unsafe extern "C" fn rand_r(seed: *mut c_uint) -> c_int {
    // libc!(libc::rand_r(seed));

    let mut z = *seed + 0x6D2B79F5;
    *seed = z;

    z = (z ^ (z >> 15)) * (z | 1);
    z ^= z + (z ^ (z >> 7)) * (z | 61);
    return (z ^ (z >> 14)) as c_int;
}