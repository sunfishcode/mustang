use libc::mode_t;
use rustix::fs::Mode;

#[no_mangle]
unsafe extern "C" fn umask(mask: mode_t) -> mode_t {
    libc!(libc::umask(mask));
    rustix::process::umask(Mode::from_raw_mode(mask)).as_raw_mode()
}
