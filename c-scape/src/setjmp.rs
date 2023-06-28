#[no_mangle]
unsafe extern "C" fn siglongjmp() {
    //libc!(libc::siglongjmp());
    unimplemented!("siglongjmp")
}

#[no_mangle]
unsafe extern "C" fn __sigsetjmp() {
    //libc!(libc::__sigsetjmp());
    unimplemented!("__sigsetjmp")
}
