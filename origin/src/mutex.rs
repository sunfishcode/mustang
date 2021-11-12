use core::cell::UnsafeCell;
use core::ptr::{null, null_mut};
use core::sync::atomic::AtomicU32;
use core::sync::atomic::Ordering::SeqCst;
use rustix::thread::{futex, FutexFlags, FutexOperation};

/// Raw mutex type backed by atomic operations and Linux `futex` calls.
///
/// The current implementation does not provide fairness, is not reentrant, and
/// is not proven correct.
///
/// This type is not movable when locked; the `new` function is `unsafe` to
/// reflect this.
// 0 => unlocked
// 1 => locked
// 2 => locked with waiters waiting
struct FutexMutex(AtomicU32);

/// This implements the same API as `lock_api::RawMutex`, except it doesn't
/// have `INIT`, so that constructing a `RawMutex` can be `unsafe`.
impl FutexMutex {
    /// Returns a new `FutexMutex`.
    ///
    /// # Safety
    ///
    /// This `FutexMutex` type is not movable when it is locked, so it should
    /// only be constructed in a place where it's never moved once it can be
    /// locked.
    #[inline]
    const unsafe fn new() -> Self {
        Self(AtomicU32::new(0))
    }

    /// Acquires this mutex, blocking the current thread until it is able to do
    /// so.
    ///
    /// This is similar to [`lock_api::RawMutex::lock`].
    ///
    /// [`lock_api::RawMutex::lock`]: https://docs.rs/lock_api/current/lock_api/trait.RawMutex.html#tymethod.lock
    #[inline]
    fn lock(&self) {
        if let Err(c) = self.0.compare_exchange(0, 1, SeqCst, SeqCst) {
            self.block(c)
        }
    }

    /// Unlocks this mutex.
    ///
    /// This is similar to [`lock_api::RawMutex::unlock`].
    ///
    /// [`lock_api::RawMutex::unlock`]: https://docs.rs/lock_api/current/lock_api/trait.RawMutex.html#tymethod.unlock
    ///
    /// # Safety
    ///
    /// This method may only be called if the mutex is held in the current
    /// context, i.e. it must be paired with a successful call to `lock` or
    /// `try_lock`.
    #[inline]
    unsafe fn unlock(&self) {
        if self.0.swap(0, SeqCst) != 1 {
            self.wake();
        }
    }

    fn block(&self, mut c: u32) {
        loop {
            // If needed, (re-)register our intent to wait.
            if c == 2
                || (match self.0.compare_exchange(1, 2, SeqCst, SeqCst) {
                    Ok(x) | Err(x) => x,
                }) != 0
            {
                // Wait until woken.
                unsafe {
                    match futex(
                        self.0.as_mut_ptr(),
                        FutexOperation::Wait,
                        FutexFlags::PRIVATE,
                        2,
                        null(),
                        null_mut(),
                        0,
                    ) {
                        Ok(_) | Err(rustix::io::Error::AGAIN) => {}
                        Err(err) => Err(err).unwrap(),
                    }
                }
            }

            // We were woken up; try to acquire the lock now.
            c = match self.0.compare_exchange(0, 2, SeqCst, SeqCst) {
                Ok(_) => break,
                Err(c) => c,
            };
        }
    }

    fn wake(&self) {
        unsafe {
            futex(
                self.0.as_mut_ptr(),
                FutexOperation::Wake,
                FutexFlags::PRIVATE,
                1,
                null(),
                null_mut(),
                0,
            )
            .unwrap();
        }
    }
}

/// A safe wrapper around `FutexMutex`, ensuring that it won't be moved while
/// locked.
pub(crate) struct Mutex<T> {
    inner: FutexMutex,
    value: UnsafeCell<T>,
}

/// # Safety
///
/// Access to the data is protected by the mutex.
unsafe impl<T> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    /// Constructs a new mutex with the given value.
    pub const fn new(value: T) -> Self {
        Mutex {
            inner: unsafe { FutexMutex::new() },
            value: UnsafeCell::new(value),
        }
    }

    /// Call `fun` with the lock held.
    ///
    /// This ensures that the lock isn't moved while locked.
    pub fn lock<R, F: FnOnce(&mut T) -> R>(&self, fun: F) -> R {
        // Since the futex is guarenteed to be unlocked by the end of the
        // function, the mutex can be moved freely outside of it.
        unsafe {
            self.inner.lock();
            let value = &mut *self.value.get();
            let res = fun(value);
            self.inner.unlock();
            res
        }
    }
}
