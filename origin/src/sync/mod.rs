//! `Mutex`, `RwLock`, and `Condvar`.
//!
//! The main implementation in in this module is derived from related source
//! files in std. We need our own copy here rather than just using what's in
//! std, because we need to be no_std, so that std can use us. And we use
//! rustix for making syscalls instead of libc.

// The following is derived from Rust's
// library/std/src/sys/unix/locks/mod.rs at revision
// 6fd7e9010db6be7605241c39eab7c5078ee2d5bd.

#![allow(missing_docs)]

// Re-export the `MovableCondvar` from std directly.
pub use condvar::MovableCondvar as Condvar;

// Export convenient `Mutex` and `RwLock` types.
pub type Mutex<T> = lock_api::Mutex<RawMutex, T>;
pub type RwLock<T> = lock_api::RwLock<RawRwLock, T>;

// std's implementation code.
mod condvar;
mod futex;
mod futex_rwlock;
mod wait_wake;

// Use the raw lock types from std's implementation.
use futex::MovableMutex;
use futex_rwlock::MovableRwLock;

// Encapsulate the std lock types to hide this detail.
#[repr(transparent)]
pub struct RawMutex(MovableMutex);
#[repr(transparent)]
pub struct RawRwLock(MovableRwLock);

// Implement the raw lock traits for our wrappers.

unsafe impl lock_api::RawMutex for RawMutex {
    type GuardMarker = lock_api::GuardNoSend;

    const INIT: Self = Self(MovableMutex::new());

    fn lock(&self) {
        self.0.lock()
    }

    fn try_lock(&self) -> bool {
        self.0.try_lock()
    }

    unsafe fn unlock(&self) {
        self.0.unlock()
    }
}

unsafe impl lock_api::RawRwLock for RawRwLock {
    type GuardMarker = lock_api::GuardNoSend;

    const INIT: Self = Self(MovableRwLock::new());

    fn lock_shared(&self) {
        self.0.read()
    }

    fn try_lock_shared(&self) -> bool {
        self.0.try_read()
    }

    unsafe fn unlock_shared(&self) {
        self.0.read_unlock()
    }

    fn lock_exclusive(&self) {
        self.0.write()
    }

    fn try_lock_exclusive(&self) -> bool {
        self.0.try_write()
    }

    unsafe fn unlock_exclusive(&self) {
        self.0.write_unlock()
    }
}
