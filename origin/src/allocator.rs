use crate::raw_mutex::Mutex;
use dlmalloc::Dlmalloc;
use std::alloc::{GlobalAlloc, Layout};

pub struct GlobalAllocator {
    inner: Mutex<Dlmalloc>,
}

impl GlobalAllocator {
    pub const fn new() -> Self {
        GlobalAllocator {
            inner: unsafe { Mutex::new(Dlmalloc::new()) },
        }
    }
}

unsafe impl GlobalAlloc for GlobalAllocator {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.inner.lock().malloc(layout.size(), layout.align())
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.inner.lock().free(ptr, layout.size(), layout.align())
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        self.inner.lock().calloc(layout.size(), layout.align())
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        self.inner
            .lock()
            .realloc(ptr, layout.size(), layout.align(), new_size)
    }
}
