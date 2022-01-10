//! Utilities to check against C signatures, when enabled.

/// casts the given list of pointers to the type used by the given expression,
/// while verifying that the layout of the pointee stays the same after the cast
macro_rules! checked_cast {
    ($expr:expr, $ptr:ident $( , $next:ident )* ) => {{
        let result;
        let target_ptr = {
            let $ptr = $ptr.cast();
            result = checked_cast!($expr $( , $next )* );
            $ptr
        };
        // uses the fact that the compiler checks for size equality,
        // when transmuting between types
        let target = Pad::new(core::ptr::read(target_ptr));
        let size_check = core::mem::transmute(core::ptr::read($ptr));
        target.compare_size(size_check);
        let align_check = core::mem::transmute(Pad::new(core::ptr::read($ptr)));
        target.compare_alignment(align_check);
        result
    }};
    ($expr:expr) => {
        $expr
    }
}

macro_rules! libc {
    ($e:expr) => {
        // TODO: Implement actually using libc. Right now this is just a
        // signature check.
        #[allow(unreachable_code)]
        if false {
            #[allow(unused_imports)]
            use crate::use_libc::*;
            // TODO: `dlopen` libc, `dlsym` the function, and call it...
            return $e;
        }
    };
}

#[cfg(feature = "threads")]
macro_rules! libc_type {
    ($name:ident, $libc:ident) => {
        #[cfg(test)]
        static_assertions::const_assert_eq!(
            core::mem::size_of::<$name>(),
            core::mem::size_of::<libc::$libc>()
        );
        #[cfg(test)]
        static_assertions::const_assert_eq!(
            core::mem::align_of::<$name>(),
            core::mem::align_of::<libc::$libc>()
        );
    };
}

// a struct that adds align_of<T> padding bytes to type T
//
// based of the rules of C struct alignment:
// align_of<Pad<T>> == max(align_of<T>, align_of<u8>) == align_of<T>,
// size_of<Pad<T>> / align_of<Pad<T>>
// == ciel( (size_of<T> + size_of<u8>) / align_of<Pad<T>>)
// and size_of<T> is divisible by align_of<T>
// therefore size_of<Pad<T>> == size_of<T> + align_of<T>
#[repr(C)]
pub(crate) struct Pad<T> {
    field: T,
    force_padding: u8,
}

impl<T> Pad<T> {
    pub unsafe fn new(v: T) -> Self {
        Pad {
            field: v,
            force_padding: 0,
        }
    }
    // used to check that size_of<T> == size_of<U> using transmute,
    pub fn compare_size(&self, _v: T) {}
    // used to check that size_of<Pad<T>> == size_of<Pad<U>> using transmute,
    // and since size_of<Pad<T>> == size_of<T> + align_of<T>,
    // if size_of<T> == size_of<U> then align_of<T> == align_of<U>
    pub fn compare_alignment(&self, _pad: Pad<T>) {}
}
