//! The following is derived from Rust's
//! library/std/src/sys/unix/locks/futex_condvar.rs at revision
//! 98815742cf2e914ee0d7142a02322cf939c47834.

use super::wait_wake::{futex_wait, futex_wake, futex_wake_all};
use crate::sync::RawMutex;
use core::sync::atomic::{AtomicU32, Ordering::Relaxed};
use core::time::Duration;
use lock_api::RawMutex as _;

pub type MovableCondvar = Condvar;

pub struct Condvar {
    // The value of this atomic is simply incremented on every notification.
    // This is used by `.wait()` to not miss any notifications after
    // unlocking the mutex and before waiting for notifications.
    futex: AtomicU32,
}

impl Condvar {
    #[inline]
    pub const fn new() -> Self {
        Self {
            futex: AtomicU32::new(0),
        }
    }

    #[inline]
    pub unsafe fn init(&mut self) {}

    #[inline]
    pub unsafe fn destroy(&self) {}

    // All the memory orderings here are `Relaxed`,
    // because synchronization is done by unlocking and locking the mutex.

    pub fn notify_one(&self) {
        self.futex.fetch_add(1, Relaxed);
        futex_wake(&self.futex);
    }

    pub fn notify_all(&self) {
        self.futex.fetch_add(1, Relaxed);
        futex_wake_all(&self.futex);
    }

    pub unsafe fn wait(&self, mutex: &RawMutex) {
        self.wait_optional_timeout(mutex, None);
    }

    pub unsafe fn wait_timeout(&self, mutex: &RawMutex, timeout: Duration) -> bool {
        self.wait_optional_timeout(mutex, Some(timeout))
    }

    unsafe fn wait_optional_timeout(&self, mutex: &RawMutex, timeout: Option<Duration>) -> bool {
        // Examine the notification counter _before_ we unlock the mutex.
        let futex_value = self.futex.load(Relaxed);

        // Unlock the mutex before going to sleep.
        mutex.unlock();

        // Wait, but only if there hasn't been any
        // notification since we unlocked the mutex.
        let r = futex_wait(&self.futex, futex_value, timeout);

        // Lock the mutex again.
        mutex.lock();

        r
    }
}
