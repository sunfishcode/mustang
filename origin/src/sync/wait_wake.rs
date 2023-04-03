//! The following is derived from Rust's
//! library/std/src/sys/unix/futex.rs at revision
//! f3579268372723bc4ff7b76090c090aa7b9e6a3a.

use core::sync::atomic::AtomicU32;
use core::time::Duration;
use rustix::thread::{FutexFlags, FutexOperation};
use rustix::time::{ClockId, Timespec};

/// Wait for a futex_wake operation to wake us.
///
/// Returns directly if the futex doesn't hold the expected value.
///
/// Returns false on timeout, and true in all other cases.
pub fn futex_wait(futex: &AtomicU32, expected: u32, timeout: Option<Duration>) -> bool {
    use core::ptr::{null, null_mut};
    use core::sync::atomic::Ordering::Relaxed;

    // Calculate the timeout as an absolute timespec.
    //
    // Overflows are rounded up to an infinite timeout (None).
    let timespec = timeout.and_then(|d| {
        Some({
            let now = rustix::time::clock_gettime(ClockId::Monotonic);
            let plus = Duration::new(now.tv_sec as u64, now.tv_nsec as _).checked_add(d)?;
            Timespec {
                tv_sec: plus.as_secs() as i64,
                tv_nsec: plus.subsec_nanos() as _,
            }
        })
    });

    loop {
        // No need to wait if the value already changed.
        if futex.load(Relaxed) != expected {
            return true;
        }

        let r = unsafe {
            // Use FUTEX_WAIT_BITSET rather than FUTEX_WAIT to be able to give an
            // absolute time rather than a relative time.
            rustix::thread::futex(
                futex.as_ptr(),
                FutexOperation::WaitBitset,
                FutexFlags::PRIVATE,
                expected,
                timespec.as_ref().map_or(null(), |t| t as *const _),
                null_mut(),
                !0u32, // A full bitmask, to make it behave like a regular FUTEX_WAIT.
            )
        };

        match r {
            Err(rustix::io::Errno::TIMEDOUT) => return false,
            Err(rustix::io::Errno::INTR) => continue,
            _ => return true,
        }
    }
}

/// Wake up one thread that's blocked on futex_wait on this futex.
///
/// Returns true if this actually woke up such a thread,
/// or false if no thread was waiting on this futex.
pub fn futex_wake(futex: &AtomicU32) -> bool {
    use core::ptr::{null, null_mut};
    unsafe {
        rustix::thread::futex(
            futex.as_ptr(),
            FutexOperation::Wake,
            FutexFlags::PRIVATE,
            1,
            null(),
            null_mut(),
            0,
        )
        .is_ok()
    }
}

/// Wake up all threads that are waiting on futex_wait on this futex.
pub fn futex_wake_all(futex: &AtomicU32) {
    use core::ptr::{null, null_mut};
    unsafe {
        rustix::thread::futex(
            futex.as_ptr(),
            FutexOperation::Wake,
            FutexFlags::PRIVATE,
            i32::MAX as u32,
            null(),
            null_mut(),
            0,
        )
        .ok();
    }
}
