//! Utilities to check against C signatures.

/// Cast the given pointer to another type, similarly to calling `cast()` on
/// it, while verifying that the layout of the pointee stays the same after the
/// cast.
macro_rules! checked_cast {
    ($ptr:ident) => {{
        let target_ptr = $ptr.cast();
        let target = crate::use_libc::Pad::new(core::ptr::read(target_ptr));

        // Uses the fact that the compiler checks for size equality,
        // when transmuting between types.
        let size_check = core::mem::transmute(core::ptr::read($ptr));
        target.compare_size(size_check);
        let align_check = core::mem::transmute(crate::use_libc::Pad::new(core::ptr::read($ptr)));
        target.compare_alignment(align_check);

        target_ptr
    }};
}

/// A macro for ensuring that the `libc` crate signature for a function matches
/// the signature that c-scape is using.
///
/// # Example
///
/// ```rust,no_run
/// #[no_mangle]
/// unsafe extern "C" fn strlen(s: *const c_char) -> usize {
///    libc!(libc::strlen(s));
///    ...
/// ```
///
/// This will elicit a compile-time error if the signature doesn't match.
macro_rules! libc {
    ($e:expr) => {
        #[allow(unreachable_code)]
        #[allow(clippy::diverging_sub_expression)]
        if false {
            #[allow(unused_imports)]
            use crate::use_libc::*;
            return $e;
        }
    };
}

/// This is used to check that a `c-scape` type matches the layout of the
/// corresponding libc type.
#[cfg(all(target_vendor = "mustang", feature = "threads"))]
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

/// A struct that adds `align_of<T>` padding bytes to type `T`.
///
/// Based on the rules of C struct alignment:
/// * `align_of<Pad<T>> == max(align_of<T>, align_of<u8>) == align_of<T>`
/// * `size_of<Pad<T>> / align_of<Pad<T>> == ciel( (size_of<T> + size_of<u8>) /
///   align_of<Pad<T>>)`
/// * `size_of<T> % align_of<T> == 0`
///
/// Therefore `size_of<Pad<T>> == size_of<T> + align_of<T>`.
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

    /// Used to check that `size_of<T> == size_of<U>` with `transmute`.
    pub fn compare_size(&self, _v: T) {}

    /// Used to check that `size_of<Pad<T>> == size_of<Pad<U>>` with
    /// `transmute`.
    ///
    /// Since `size_of<Pad<T>> == size_of<T> + align_of<T>`,
    /// if `size_of<T> == size_of<U>` then `align_of<T> == align_of<U>`.
    pub fn compare_alignment(&self, _pad: Pad<T>) {}
}
