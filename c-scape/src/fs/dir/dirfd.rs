use rustix::fd::{AsFd, AsRawFd};

use libc::{c_int, c_void};

use super::CScapeDir;

#[no_mangle]
unsafe extern "C" fn dirfd(dir: *mut c_void) -> c_int {
    libc!(libc::dirfd(dir.cast()));

    let dir = dir.cast::<CScapeDir>();
    (*dir).fd.as_fd().as_raw_fd()
}
