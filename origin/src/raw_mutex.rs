use lock_api::{GuardSend, Mutex as GenericMutex, MutexGuard as GenericMutexGuard, RawMutex};
use rsix::thread::{futex, FutexFlags, FutexOperation};
use std::ops::{Deref, DerefMut};
use std::ptr::{null, null_mut};
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::SeqCst;

/// Raw mutex type backed by atomic operations and Linux `futex` calls.
///
/// The current implementation does not provide fairness, is not reentrant,
/// and is not proven correct.
///
/// #Safety:
/// This type is not movable when locked; the `new` function is `unsafe` to
/// reflect this.
/// The `INIT` constant only exists to implement `lock_api::RawMutex`,
/// and should not be used to construct the type directly.
///
/// it is safe to use outside this module, since the only way to construct it,
/// is inside a `Mutex`, which returns a `MutexGuard` while locked.
/// While the `MutexGuard` is in scope, the `Mutex` is borrowed,
/// and so in safe rust cannot be accidently moved.
// 0 => unlocked
// 1 => locked
// 2 => locked with waiters waiting
struct FutexMutex(AtomicU32);

unsafe impl RawMutex for FutexMutex {
    const INIT: Self = unsafe { FutexMutex::new() };

    type GuardMarker = GuardSend;

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

    /// Attempts to acquire this mutex without blocking. Returns `true` if the
    /// lock was successfully acquired and `false` otherwise.
    ///
    /// This is similar to [`lock_api::RawMutex::try_lock`].
    ///
    /// [`lock_api::RawMutex::try_lock`]: https://docs.rs/lock_api/current/lock_api/trait.RawMutex.html#tymethod.try_lock
    #[inline]
    fn try_lock(&self) -> bool {
        self.0.compare_exchange(0, 1, SeqCst, SeqCst).is_ok()
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

    fn is_locked(&self) -> bool {
        self.0.load(SeqCst) == 0
    }
}

/// This implements the same API as `lock_api::RawMutex`, except it doesn't
/// have `INIT`, so that constructing a `FutexMutex` can be `unsafe`.
impl FutexMutex {
    /// Returns a new `FutexMutex`.
    ///
    /// # Safety
    ///
    /// This `FutexMutex` type is not movable when it is locked, so it should
    /// only be constructed in a place where it's never moved once it can be
    /// locked.
    #[inline]
    pub(crate) const unsafe fn new() -> Self {
        Self(AtomicU32::new(0))
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
                        Ok(_) | Err(rsix::io::Error::AGAIN) => {}
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

pub(crate) struct Mutex<T> {
    inner: GenericMutex<FutexMutex, T>,
}

/// this types wraps the mutex guard,
/// which normally provides direct access to the underlying RawMutex.
/// This allows the FutexMutex to remain private to this module,
/// thus ensuring that it will only be constructed via `Mutex`.
pub(crate) struct MutexGuard<'a, T> {
    inner: GenericMutexGuard<'a, FutexMutex, T>,
}

impl<T> Mutex<T> {
    pub const unsafe fn new(value: T) -> Self {
        Mutex {
            inner: GenericMutex::const_new(FutexMutex::new(), value),
        }
    }

    pub fn lock(&self) -> MutexGuard<'_, T> {
        MutexGuard {
            inner: self.inner.lock(),
        }
    }
}

impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, T> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
