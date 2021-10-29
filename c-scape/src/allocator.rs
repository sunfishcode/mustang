// We need to enable some global allocator, because `c-scape` implements
// `malloc` using the Rust global allocator, and the default Rust global
// allocator uses `malloc`.
#[cfg(not(any(feature = "dlmalloc", feature = "wee_alloc")))]
compile_error!("Either feature \"dlmalloc\" or \"wee_alloc\" must be enabled for this crate.");

#[cfg(all(feature = "dlmalloc", not(feature = "threads")))]
#[global_allocator]
static GLOBAL_ALLOCATOR: dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;

#[cfg(all(feature = "wee_alloc", not(feature = "threads")))]
#[global_allocator]
static GLOBAL_ALLOCATOR: wee_alloc::WeeAlloc<'static> = wee_alloc::WeeAlloc::INIT;

#[cfg(all(any(feature = "dlmalloc", feature = "wee_alloc"), feature = "threads"))]
#[global_allocator]
static GLOBAL_ALLOCATOR: inner::GlobalAllocator = unsafe { inner::GlobalAllocator::new() };

#[cfg(all(any(feature = "dlmalloc", feature = "wee_alloc"), feature = "threads"))]
mod inner {
    use crate::RawMutex;

    use std::alloc::Layout;
    #[cfg(feature = "dlmalloc")]
    use std::cell::UnsafeCell;

    pub struct GlobalAllocator {
        pub(crate) mutex: RawMutex,
        #[cfg(feature = "dlmalloc")]
        inner: UnsafeCell<dlmalloc::Dlmalloc>,
        #[cfg(feature = "wee_alloc")]
        inner: wee_alloc::WeeAlloc<'static>,
    }

    #[cfg(feature = "dlmalloc")]
    unsafe impl Sync for GlobalAllocator {}

    impl GlobalAllocator {
        pub(crate) const unsafe fn new() -> Self {
            GlobalAllocator {
                mutex: RawMutex::new(),
                #[cfg(feature = "dlmalloc")]
                inner: UnsafeCell::new(dlmalloc::Dlmalloc::new()),
                #[cfg(feature = "wee_alloc")]
                inner: wee_alloc::WeeAlloc::INIT,
            }
        }
    }

    #[macro_export]
    macro_rules! with_mutex {
        ( $mutex:expr, $exp:expr ) => {{
            $mutex.lock();
            let res = $exp;
            $mutex.unlock();
            res
        }};
    }

    #[cfg(feature = "dlmalloc")]
    unsafe impl std::alloc::GlobalAlloc for GlobalAllocator {
        #[inline]
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            with_mutex!(
                self.mutex,
                (&mut *self.inner.get()).malloc(layout.size(), layout.align())
            )
        }

        #[inline]
        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
            with_mutex!(
                self.mutex,
                (&mut *self.inner.get()).free(ptr, layout.size(), layout.align())
            )
        }

        #[inline]
        unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
            with_mutex!(
                self.mutex,
                (&mut *self.inner.get()).calloc(layout.size(), layout.align())
            )
        }

        #[inline]
        unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
            with_mutex!(
                self.mutex,
                (&mut *self.inner.get()).realloc(ptr, layout.size(), layout.align(), new_size)
            )
        }
    }

    #[cfg(feature = "wee_alloc")]
    unsafe impl std::alloc::GlobalAlloc for GlobalAllocator {
        #[inline]
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            with_mutex!(self.mutex, self.inner.alloc(layout))
        }

        #[inline]
        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
            with_mutex!(self.mutex, self.inner.dealloc(ptr, layout))
        }

        #[inline]
        unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
            with_mutex!(self.mutex, self.inner.alloc_zeroed(layout))
        }

        #[inline]
        unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
            with_mutex!(self.mutex, self.inner.realloc(ptr, layout, new_size))
        }
    }
}
