use core::sync::atomic::Ordering::{AcqRel, Acquire, Relaxed, Release};
use core::sync::atomic::{AtomicU16, AtomicU32, AtomicU64, AtomicU8};

#[no_mangle]
unsafe extern "C" fn __aarch64_ldadd1_acq(x: u8, p: *mut AtomicU8) -> u8 {
    (*p).fetch_add(x, Acquire)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldadd1_acq_rel(x: u8, p: *mut AtomicU8) -> u8 {
    (*p).fetch_add(x, AcqRel)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldadd1_rel(x: u8, p: *mut AtomicU8) -> u8 {
    (*p).fetch_add(x, Release)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldadd1_relax(x: u8, p: *mut AtomicU8) -> u8 {
    (*p).fetch_add(x, Relaxed)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldadd2_acq(x: u16, p: *mut AtomicU16) -> u16 {
    (*p).fetch_add(x, Acquire)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldadd2_acq_rel(x: u16, p: *mut AtomicU16) -> u16 {
    (*p).fetch_add(x, AcqRel)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldadd2_rel(x: u16, p: *mut AtomicU16) -> u16 {
    (*p).fetch_add(x, Release)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldadd2_relax(x: u16, p: *mut AtomicU16) -> u16 {
    (*p).fetch_add(x, Relaxed)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldadd4_acq(x: u32, p: *mut AtomicU32) -> u32 {
    (*p).fetch_add(x, Acquire)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldadd4_acq_rel(x: u32, p: *mut AtomicU32) -> u32 {
    (*p).fetch_add(x, AcqRel)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldadd4_rel(x: u32, p: *mut AtomicU32) -> u32 {
    (*p).fetch_add(x, Release)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldadd4_relax(x: u32, p: *mut AtomicU32) -> u32 {
    (*p).fetch_add(x, Relaxed)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldadd8_acq(x: u64, p: *mut AtomicU64) -> u64 {
    (*p).fetch_add(x, Acquire)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldadd8_acq_rel(x: u64, p: *mut AtomicU64) -> u64 {
    (*p).fetch_add(x, AcqRel)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldadd8_rel(x: u64, p: *mut AtomicU64) -> u64 {
    (*p).fetch_add(x, Release)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldadd8_relax(x: u64, p: *mut AtomicU64) -> u64 {
    (*p).fetch_add(x, Relaxed)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldeor1_acq(x: u8, p: *mut AtomicU8) -> u8 {
    (*p).fetch_xor(x, Acquire)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldeor1_acq_rel(x: u8, p: *mut AtomicU8) -> u8 {
    (*p).fetch_xor(x, AcqRel)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldeor1_rel(x: u8, p: *mut AtomicU8) -> u8 {
    (*p).fetch_xor(x, Release)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldeor1_relax(x: u8, p: *mut AtomicU8) -> u8 {
    (*p).fetch_xor(x, Relaxed)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldeor2_acq(x: u16, p: *mut AtomicU16) -> u16 {
    (*p).fetch_xor(x, Acquire)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldeor2_acq_rel(x: u16, p: *mut AtomicU16) -> u16 {
    (*p).fetch_xor(x, AcqRel)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldeor2_rel(x: u16, p: *mut AtomicU16) -> u16 {
    (*p).fetch_xor(x, Release)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldeor2_relax(x: u16, p: *mut AtomicU16) -> u16 {
    (*p).fetch_xor(x, Relaxed)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldeor4_acq(x: u32, p: *mut AtomicU32) -> u32 {
    (*p).fetch_xor(x, Acquire)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldeor4_acq_rel(x: u32, p: *mut AtomicU32) -> u32 {
    (*p).fetch_xor(x, AcqRel)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldeor4_rel(x: u32, p: *mut AtomicU32) -> u32 {
    (*p).fetch_xor(x, Release)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldeor4_relax(x: u32, p: *mut AtomicU32) -> u32 {
    (*p).fetch_xor(x, Relaxed)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldeor8_acq(x: u64, p: *mut AtomicU64) -> u64 {
    (*p).fetch_xor(x, Acquire)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldeor8_acq_rel(x: u64, p: *mut AtomicU64) -> u64 {
    (*p).fetch_xor(x, AcqRel)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldeor8_rel(x: u64, p: *mut AtomicU64) -> u64 {
    (*p).fetch_xor(x, Release)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldeor8_relax(x: u64, p: *mut AtomicU64) -> u64 {
    (*p).fetch_xor(x, Relaxed)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldclr1_acq(x: u8, p: *mut AtomicU8) -> u8 {
    (*p).fetch_and(!x, Acquire)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldclr1_acq_rel(x: u8, p: *mut AtomicU8) -> u8 {
    (*p).fetch_and(!x, AcqRel)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldclr1_rel(x: u8, p: *mut AtomicU8) -> u8 {
    (*p).fetch_and(!x, Release)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldclr1_relax(x: u8, p: *mut AtomicU8) -> u8 {
    (*p).fetch_and(!x, Relaxed)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldclr2_acq(x: u16, p: *mut AtomicU16) -> u16 {
    (*p).fetch_and(!x, Acquire)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldclr2_acq_rel(x: u16, p: *mut AtomicU16) -> u16 {
    (*p).fetch_and(!x, AcqRel)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldclr2_rel(x: u16, p: *mut AtomicU16) -> u16 {
    (*p).fetch_and(!x, Release)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldclr2_relax(x: u16, p: *mut AtomicU16) -> u16 {
    (*p).fetch_and(!x, Relaxed)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldclr4_acq(x: u32, p: *mut AtomicU32) -> u32 {
    (*p).fetch_and(!x, Acquire)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldclr4_acq_rel(x: u32, p: *mut AtomicU32) -> u32 {
    (*p).fetch_and(!x, AcqRel)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldclr4_rel(x: u32, p: *mut AtomicU32) -> u32 {
    (*p).fetch_and(!x, Release)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldclr4_relax(x: u32, p: *mut AtomicU32) -> u32 {
    (*p).fetch_and(!x, Relaxed)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldclr8_acq(x: u64, p: *mut AtomicU64) -> u64 {
    (*p).fetch_and(!x, Acquire)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldclr8_acq_rel(x: u64, p: *mut AtomicU64) -> u64 {
    (*p).fetch_and(!x, AcqRel)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldclr8_rel(x: u64, p: *mut AtomicU64) -> u64 {
    (*p).fetch_and(!x, Release)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldclr8_relax(x: u64, p: *mut AtomicU64) -> u64 {
    (*p).fetch_and(!x, Relaxed)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_swp1_acq(x: u8, p: *mut AtomicU8) -> u8 {
    (*p).swap(x, Acquire)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_swp1_acq_rel(x: u8, p: *mut AtomicU8) -> u8 {
    (*p).swap(x, AcqRel)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_swp1_rel(x: u8, p: *mut AtomicU8) -> u8 {
    (*p).swap(x, Release)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_swp1_relax(x: u8, p: *mut AtomicU8) -> u8 {
    (*p).swap(x, Relaxed)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_swp2_acq(x: u16, p: *mut AtomicU16) -> u16 {
    (*p).swap(x, Acquire)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_swp2_acq_rel(x: u16, p: *mut AtomicU16) -> u16 {
    (*p).swap(x, AcqRel)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_swp2_rel(x: u16, p: *mut AtomicU16) -> u16 {
    (*p).swap(x, Release)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_swp2_relax(x: u16, p: *mut AtomicU16) -> u16 {
    (*p).swap(x, Relaxed)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_swp4_acq(x: u32, p: *mut AtomicU32) -> u32 {
    (*p).swap(x, Acquire)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_swp4_acq_rel(x: u32, p: *mut AtomicU32) -> u32 {
    (*p).swap(x, AcqRel)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_swp4_rel(x: u32, p: *mut AtomicU32) -> u32 {
    (*p).swap(x, Release)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_swp4_relax(x: u32, p: *mut AtomicU32) -> u32 {
    (*p).swap(x, Relaxed)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_swp8_acq(x: u64, p: *mut AtomicU64) -> u64 {
    (*p).swap(x, Acquire)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_swp8_acq_rel(x: u64, p: *mut AtomicU64) -> u64 {
    (*p).swap(x, AcqRel)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_swp8_rel(x: u64, p: *mut AtomicU64) -> u64 {
    (*p).swap(x, Release)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_swp8_relax(x: u64, p: *mut AtomicU64) -> u64 {
    (*p).swap(x, Relaxed)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_cas1_acq(x: u8, y: u8, p: *mut AtomicU8) -> u8 {
    match (*p).compare_exchange(x, y, Acquire, Acquire) {
        Ok(r) | Err(r) => r,
    }
}

#[no_mangle]
unsafe extern "C" fn __aarch64_cas1_acq_rel(x: u8, y: u8, p: *mut AtomicU8) -> u8 {
    match (*p).compare_exchange(x, y, AcqRel, Acquire) {
        Ok(r) | Err(r) => r,
    }
}

#[no_mangle]
unsafe extern "C" fn __aarch64_cas1_rel(x: u8, y: u8, p: *mut AtomicU8) -> u8 {
    match (*p).compare_exchange(x, y, Release, Relaxed) {
        Ok(r) | Err(r) => r,
    }
}

#[no_mangle]
unsafe extern "C" fn __aarch64_cas1_relax(x: u8, y: u8, p: *mut AtomicU8) -> u8 {
    match (*p).compare_exchange(x, y, Relaxed, Relaxed) {
        Ok(r) | Err(r) => r,
    }
}

#[no_mangle]
unsafe extern "C" fn __aarch64_cas2_acq(x: u16, y: u16, p: *mut AtomicU16) -> u16 {
    match (*p).compare_exchange(x, y, Acquire, Acquire) {
        Ok(r) | Err(r) => r,
    }
}

#[no_mangle]
unsafe extern "C" fn __aarch64_cas2_acq_rel(x: u16, y: u16, p: *mut AtomicU16) -> u16 {
    match (*p).compare_exchange(x, y, AcqRel, Acquire) {
        Ok(r) | Err(r) => r,
    }
}

#[no_mangle]
unsafe extern "C" fn __aarch64_cas2_rel(x: u16, y: u16, p: *mut AtomicU16) -> u16 {
    match (*p).compare_exchange(x, y, Release, Relaxed) {
        Ok(r) | Err(r) => r,
    }
}

#[no_mangle]
unsafe extern "C" fn __aarch64_cas2_relax(x: u16, y: u16, p: *mut AtomicU16) -> u16 {
    match (*p).compare_exchange(x, y, Relaxed, Relaxed) {
        Ok(r) | Err(r) => r,
    }
}

#[no_mangle]
unsafe extern "C" fn __aarch64_cas4_acq(x: u32, y: u32, p: *mut AtomicU32) -> u32 {
    match (*p).compare_exchange(x, y, Acquire, Acquire) {
        Ok(r) | Err(r) => r,
    }
}

#[no_mangle]
unsafe extern "C" fn __aarch64_cas4_acq_rel(x: u32, y: u32, p: *mut AtomicU32) -> u32 {
    match (*p).compare_exchange(x, y, AcqRel, Acquire) {
        Ok(r) | Err(r) => r,
    }
}

#[no_mangle]
unsafe extern "C" fn __aarch64_cas4_rel(x: u32, y: u32, p: *mut AtomicU32) -> u32 {
    match (*p).compare_exchange(x, y, Release, Relaxed) {
        Ok(r) | Err(r) => r,
    }
}

#[no_mangle]
unsafe extern "C" fn __aarch64_cas4_relax(x: u32, y: u32, p: *mut AtomicU32) -> u32 {
    match (*p).compare_exchange(x, y, Relaxed, Relaxed) {
        Ok(r) | Err(r) => r,
    }
}

#[no_mangle]
unsafe extern "C" fn __aarch64_cas8_acq(x: u64, y: u64, p: *mut AtomicU64) -> u64 {
    match (*p).compare_exchange(x, y, Acquire, Acquire) {
        Ok(r) | Err(r) => r,
    }
}

#[no_mangle]
unsafe extern "C" fn __aarch64_cas8_acq_rel(x: u64, y: u64, p: *mut AtomicU64) -> u64 {
    match (*p).compare_exchange(x, y, AcqRel, Acquire) {
        Ok(r) | Err(r) => r,
    }
}

#[no_mangle]
unsafe extern "C" fn __aarch64_cas8_rel(x: u64, y: u64, p: *mut AtomicU64) -> u64 {
    match (*p).compare_exchange(x, y, Release, Relaxed) {
        Ok(r) | Err(r) => r,
    }
}

#[no_mangle]
unsafe extern "C" fn __aarch64_cas8_relax(x: u64, y: u64, p: *mut AtomicU64) -> u64 {
    match (*p).compare_exchange(x, y, Relaxed, Relaxed) {
        Ok(r) | Err(r) => r,
    }
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldset1_acq(x: u8, p: *mut AtomicU8) -> u8 {
    (*p).fetch_or(x, Acquire)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldset1_acq_rel(x: u8, p: *mut AtomicU8) -> u8 {
    (*p).fetch_or(x, AcqRel)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldset1_rel(x: u8, p: *mut AtomicU8) -> u8 {
    (*p).fetch_or(x, Release)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldset1_relax(x: u8, p: *mut AtomicU8) -> u8 {
    (*p).fetch_or(x, Relaxed)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldset2_acq(x: u16, p: *mut AtomicU16) -> u16 {
    (*p).fetch_or(x, Acquire)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldset2_acq_rel(x: u16, p: *mut AtomicU16) -> u16 {
    (*p).fetch_or(x, AcqRel)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldset2_rel(x: u16, p: *mut AtomicU16) -> u16 {
    (*p).fetch_or(x, Release)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldset2_relax(x: u16, p: *mut AtomicU16) -> u16 {
    (*p).fetch_or(x, Relaxed)
}
#[no_mangle]
unsafe extern "C" fn __aarch64_ldset4_acq(x: u32, p: *mut AtomicU32) -> u32 {
    (*p).fetch_or(x, Acquire)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldset4_acq_rel(x: u32, p: *mut AtomicU32) -> u32 {
    (*p).fetch_or(x, AcqRel)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldset4_rel(x: u32, p: *mut AtomicU32) -> u32 {
    (*p).fetch_or(x, Release)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldset4_relax(x: u32, p: *mut AtomicU32) -> u32 {
    (*p).fetch_or(x, Relaxed)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldset8_acq(x: u64, p: *mut AtomicU64) -> u64 {
    (*p).fetch_or(x, Acquire)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldset8_acq_rel(x: u64, p: *mut AtomicU64) -> u64 {
    (*p).fetch_or(x, AcqRel)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldset8_rel(x: u64, p: *mut AtomicU64) -> u64 {
    (*p).fetch_or(x, Release)
}

#[no_mangle]
unsafe extern "C" fn __aarch64_ldset8_relax(x: u64, p: *mut AtomicU64) -> u64 {
    (*p).fetch_or(x, Relaxed)
}
