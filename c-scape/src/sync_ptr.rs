/// A const pointer to `T` that implements [`Sync`].
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SyncConstPtr<T>(*const T);
unsafe impl<T> Sync for SyncConstPtr<T> {}

impl<T> SyncConstPtr<T> {
    /// Creates a `SyncConstPointer` from a raw pointer.
    ///
    /// Behavior is undefined if `ptr` is actually not
    /// safe to share across threads.
    pub const unsafe fn new(ptr: *const T) -> Self {
        Self(ptr)
    }
}

/// A mut pointer to `T` that implements [`Sync`].
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SyncMutPtr<T>(*mut T);
unsafe impl<T> Sync for SyncMutPtr<T> {}

impl<T> SyncMutPtr<T> {
    /// Creates a `SyncMutPtr` from a raw pointer.
    ///
    /// Behavior is undefined if `ptr` is actually not
    /// safe to share across threads.
    pub const unsafe fn new(ptr: *mut T) -> Self {
        Self(ptr)
    }
}
