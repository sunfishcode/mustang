use rustix::fd::IntoRawFd;
use rustix::pipe::PipeFlags;

use libc::c_int;

use crate::convert_res;

#[no_mangle]
unsafe extern "C" fn pipe(pipefd: *mut c_int) -> c_int {
    libc!(libc::pipe(pipefd));

    match convert_res(rustix::pipe::pipe()) {
        Some((a, b)) => {
            *pipefd = a.into_raw_fd();
            *pipefd.add(1) = b.into_raw_fd();
            0
        }
        None => -1,
    }
}

#[cfg(any(target_os = "android", target_os = "linux"))]
#[no_mangle]
unsafe extern "C" fn pipe2(pipefd: *mut c_int, flags: c_int) -> c_int {
    libc!(libc::pipe2(pipefd, flags));

    let flags = PipeFlags::from_bits(flags as _).unwrap();
    match convert_res(rustix::pipe::pipe_with(flags)) {
        Some((a, b)) => {
            *pipefd = a.into_raw_fd();
            *pipefd.add(1) = b.into_raw_fd();
            0
        }
        None => -1,
    }
}
