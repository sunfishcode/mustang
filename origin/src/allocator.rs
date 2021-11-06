use crate::mutex::Mutex;
use dlmalloc::Dlmalloc;
use std::alloc::{GlobalAlloc, Layout};

static INNER_ALLOC: Mutex<Dlmalloc> = Mutex::new(Dlmalloc::new());

/// the global allocator that should be used when targeting mustang
pub struct GlobalAllocator;

unsafe impl GlobalAlloc for GlobalAllocator {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        INNER_ALLOC.lock(|dlmalloc| dlmalloc.malloc(layout.size(), layout.align()))
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        INNER_ALLOC.lock(|dlmalloc| dlmalloc.free(ptr, layout.size(), layout.align()))
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        INNER_ALLOC.lock(|dlmalloc| dlmalloc.calloc(layout.size(), layout.align()))
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        INNER_ALLOC.lock(|dlmalloc| dlmalloc.realloc(ptr, layout.size(), layout.align(), new_size))
    }
}
