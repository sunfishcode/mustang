mod bstring;
mod string;

// <https://refspecs.linuxbase.org/LSB_4.0.0/LSB-Core-generic/LSB-Core-generic/libc---chk-fail-1.html>
#[no_mangle]
unsafe extern "C" fn __chk_fail() {
    rustix::io::write(
        rustix::stdio::stderr(),
        b"A buffer overflow has been detected.\n",
    )
    .ok();
    libc::abort();
}
