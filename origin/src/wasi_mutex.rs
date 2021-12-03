//! Minimal `Mutex` for WASI, assuming it doesn't support threads.

#![cfg(target_os = "wasi")]

use core::cell::RefCell;

pub(crate) struct Mutex<T> {
    value: RefCell<T>,
}

/// # Safety
///
/// WASI does not support threads yet.
unsafe impl<T> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    pub const fn new(value: T) -> Self {
        Mutex {
            value: RefCell::new(value),
        }
    }

    pub fn lock<R, F: FnOnce(&mut T) -> R>(&self, fun: F) -> R {
        let value = &mut *self.value.borrow_mut();
        fun(value)
    }
}
