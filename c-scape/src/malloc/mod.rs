#[cfg(not(target_arch = "riscv64"))]
use core::mem::align_of;
use core::mem::size_of;
use core::ptr;
use errno::{set_errno, Errno};
use libc::{c_int, c_void, memcpy, memset};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Tag {
    size: usize,
    align: usize,
}

/// allocates for a given layout, with a tag prepended to the allocation,
/// to keep track of said layout.
/// returns null if the allocation failed
fn tagged_alloc(type_layout: alloc::alloc::Layout) -> *mut u8 {
    if type_layout.size() == 0 {
        return ptr::null_mut();
    }

    let tag_layout = alloc::alloc::Layout::new::<Tag>();
    let tag = Tag {
        size: type_layout.size(),
        align: type_layout.align(),
    };
    if let Ok((total_layout, offset)) = tag_layout.extend(type_layout) {
        let total_ptr = unsafe { alloc::alloc::alloc(total_layout) };
        if total_ptr.is_null() {
            return total_ptr;
        }

        let tag_offset = offset - tag_layout.size();
        unsafe {
            total_ptr.wrapping_add(tag_offset).cast::<Tag>().write(tag);
            total_ptr.wrapping_add(offset).cast()
        }
    } else {
        ptr::null_mut()
    }
}

/// get the layout out of a tagged allocation
///
/// #Safety
/// the given pointer must be a non-null pointer,
/// gotten from calling `tagged_alloc`
unsafe fn get_layout(ptr: *mut u8) -> alloc::alloc::Layout {
    let tag = ptr.wrapping_sub(size_of::<Tag>()).cast::<Tag>().read();
    alloc::alloc::Layout::from_size_align_unchecked(tag.size, tag.align)
}

/// get the layout out of a tagged allocation
///
/// #Safety
/// the given pointer must be a non-null pointer,
/// gotten from calling `tagged_alloc`
unsafe fn tagged_dealloc(ptr: *mut u8) {
    let tag_layout = alloc::alloc::Layout::new::<Tag>();
    let type_layout = get_layout(ptr);
    if let Ok((total_layout, offset)) = tag_layout.extend(type_layout) {
        let total_ptr = ptr.wrapping_sub(offset);
        alloc::alloc::dealloc(total_ptr, total_layout);
    }
}

#[no_mangle]
unsafe extern "C" fn malloc(size: usize) -> *mut c_void {
    libc!(libc::malloc(size));

    // TODO: Add `max_align_t` for riscv64 to upstream libc.
    #[cfg(target_arch = "riscv64")]
    let layout = alloc::alloc::Layout::from_size_align(size, 16).unwrap();
    #[cfg(not(target_arch = "riscv64"))]
    let layout =
        alloc::alloc::Layout::from_size_align(size, align_of::<libc::max_align_t>()).unwrap();

    let ret = tagged_alloc(layout);
    if ret.is_null() {
        set_errno(Errno(libc::ENOMEM));
    }
    ret.cast()
}

#[no_mangle]
unsafe extern "C" fn realloc(old: *mut c_void, size: usize) -> *mut c_void {
    libc!(libc::realloc(old, size));

    if old.is_null() {
        malloc(size)
    } else {
        let old_layout = get_layout(old.cast());
        if old_layout.size() >= size {
            return old;
        }

        let new = malloc(size);

        memcpy(new, old, core::cmp::min(size, old_layout.size()));
        tagged_dealloc(old.cast());
        new
    }
}

#[no_mangle]
unsafe extern "C" fn calloc(nmemb: usize, size: usize) -> *mut c_void {
    libc!(libc::calloc(nmemb, size));

    let product = match nmemb.checked_mul(size) {
        Some(product) => product,
        None => {
            set_errno(Errno(libc::ENOMEM));
            return ptr::null_mut();
        }
    };

    let ptr = malloc(product);
    memset(ptr, 0, product)
}

#[no_mangle]
unsafe extern "C" fn posix_memalign(
    memptr: *mut *mut c_void,
    alignment: usize,
    size: usize,
) -> c_int {
    libc!(libc::posix_memalign(memptr, alignment, size));
    if !(alignment.is_power_of_two() && alignment % core::mem::size_of::<*const c_void>() == 0) {
        return rustix::io::Errno::INVAL.raw_os_error();
    }

    let layout = alloc::alloc::Layout::from_size_align(size, alignment).unwrap();
    let ptr = tagged_alloc(layout).cast();

    *memptr = ptr;
    0
}

#[no_mangle]
unsafe extern "C" fn free(ptr: *mut c_void) {
    libc!(libc::free(ptr));

    if ptr.is_null() {
        return;
    }

    tagged_dealloc(ptr.cast());
}
