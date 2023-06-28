use libc::c_char;

#[no_mangle]
unsafe extern "C" fn gnu_get_libc_version() -> *const c_char {
    // We're implementing the glibc ABI, but we aren't actually glibc. This
    // is used by `std` to test for certain behaviors. For now, return 2.23
    // which is a glibc version where `posix_spawn` doesn't handle `ENOENT`
    // as std wants, so it uses `fork`+`exec` instead, since we don't yet
    // implement `posix_spawn`.
    b"2.23\0".as_ptr().cast()
}
