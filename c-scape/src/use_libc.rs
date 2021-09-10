//! Utilities to check against C signatures, when enabled.

macro_rules! libc {
    ($e:expr) => {
        if true {
            use libc::*;
            // We'll probably need to `dlopen` libc and load symbols from
            // it manually.
            assert!(false, "actually calling libc is not implemented yet");
            return $e;
        }
    };
}

pub fn same_ptr<T, U>(t: *const T) -> *const U {
    assert_eq!(std::mem::size_of::<T>(), std::mem::size_of::<U>());
    assert_eq!(std::mem::align_of::<T>(), std::mem::align_of::<U>());
    t.cast::<U>()
}

pub fn same_ptr_mut<T, U>(t: *mut T) -> *mut U {
    assert_eq!(std::mem::size_of::<T>(), std::mem::size_of::<U>());
    assert_eq!(std::mem::align_of::<T>(), std::mem::align_of::<U>());
    t.cast::<U>()
}
