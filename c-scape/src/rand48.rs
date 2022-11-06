use libc::{c_double, c_long, c_ushort};

use core::cell::SyncUnsafeCell;
use core::ptr::{addr_of, addr_of_mut};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct LCongData {
    a_mult: [c_ushort; 3],
    c: c_ushort,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct RngData {
    x_subi: [c_ushort; 3],
    data: LCongData,
}

static STORAGE: SyncUnsafeCell<RngData> = SyncUnsafeCell::new(RngData {
    x_subi: [0, 0, 0],
    data: LCongData {
        a_mult: [0xe66d, 0xdeec, 0x5],
        c: 0xb,
    },
});

unsafe fn next_lcong(x_subi: *mut [c_ushort; 3], data: *const LCongData) -> u64 {
    let x: u64 = ((*x_subi)[0] as u64) | ((*x_subi)[1] as u64) << 16 | ((*x_subi)[2] as u64) << 32;
    let a: u64 = ((*data).a_mult[0] as u64)
        | ((*data).a_mult[1] as u64) << 16
        | ((*data).a_mult[2] as u64) << 32;

    let res = a * x + (*data).c as u64;
    (*x_subi)[0] = res as c_ushort;
    (*x_subi)[1] = (res >> 16) as c_ushort;
    (*x_subi)[2] = (res >> 32) as c_ushort;
    res
}

#[no_mangle]
unsafe extern "C" fn drand48() -> c_double {
    // libc!(libc::drand48());

    erand48(addr_of_mut!((*STORAGE.get()).x_subi))
}

#[no_mangle]
unsafe extern "C" fn erand48(x_subi: *mut [c_ushort; 3]) -> c_double {
    //libc!(libc::erand48(xsubi));
    const TO_DIVIDE: c_double = (2i64 << 48) as c_double;
    next_lcong(x_subi, addr_of!((*STORAGE.get()).data)) as c_double / TO_DIVIDE
}

#[no_mangle]
unsafe extern "C" fn lrand48() -> c_long {
    //libc!(libc::lrand48());

    nrand48(addr_of_mut!((*STORAGE.get()).x_subi))
}

#[no_mangle]
unsafe extern "C" fn nrand48(x_subi: *mut [c_ushort; 3]) -> c_long {
    //libc!(libc::nrand48(xsubi));

    (next_lcong(x_subi, addr_of!((*STORAGE.get()).data)) >> 17) as i64
}

#[no_mangle]
unsafe extern "C" fn mrand48() -> c_long {
    //libc!(libc::mrand48());

    jrand48(addr_of_mut!((*STORAGE.get()).x_subi))
}

#[no_mangle]
unsafe extern "C" fn jrand48(x_subi: *mut [c_ushort; 3]) -> c_long {
    //libc!(libc::jrand48(xsubi));

    // Cast to i32 so the sign extension works properly
    (next_lcong(x_subi, addr_of!((*STORAGE.get()).data)) >> 16) as i32 as c_long
}

#[no_mangle]
unsafe extern "C" fn srand48(seed: c_long) {
    //libc!(libc::srand48(seedval));

    seed48(&mut [0x330e, seed as c_ushort, (seed >> 16) as c_ushort]);
}

#[no_mangle]
unsafe extern "C" fn seed48(seed: *mut [c_ushort; 3]) -> *mut c_ushort {
    static PREV_SEED: SyncUnsafeCell<[c_ushort; 3]> = SyncUnsafeCell::new([0, 0, 0]);

    *PREV_SEED.get() = (*STORAGE.get()).x_subi;
    (*STORAGE.get()).x_subi = *seed;

    PREV_SEED.get().cast::<u16>()
}

#[no_mangle]
unsafe extern "C" fn lcong48(param: *mut [c_ushort; 7]) {
    //libc!(libc::lcong48(param));

    *STORAGE.get().cast::<[c_ushort; 7]>() = *param;
}
