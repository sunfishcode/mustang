use rustix::fd::BorrowedFd;
use rustix::fs::{AtFlags, Mode, OFlags};

use alloc::vec::Vec;
use core::ffi::CStr;

use crate::{Errno, set_errno};

use libc::{c_char, c_int};

extern "C" {
    static mut environ: *mut *mut c_char;
}

#[no_mangle]
unsafe extern "C" fn execl(
    path: *const c_char, 
    arg: *const c_char,
    mut argv: ...
) -> c_int {
    let mut vec = Vec::new();
    vec.push(arg);

    loop {
        let ptr = argv.arg::<*const c_char>();
        vec.push(ptr);
        if ptr.is_null() {
            break;
        }
    }
    
    execv(path, vec.as_ptr())
}

#[no_mangle]
pub unsafe extern "C" fn execle(
    path: *const c_char, 
    arg: *const c_char,
    mut argv: ...
) -> c_int {
    let mut vec = Vec::new();
    vec.push(arg);

    let envp = loop {
        let ptr = argv.arg::<*const c_char>();
        vec.push(ptr);
        if ptr.is_null() {
            break argv.arg::<*const *const c_char>();
        }
    };

    execve(path, vec.as_ptr(), envp)
}

#[no_mangle]
unsafe extern "C" fn execlp(
    file: *const c_char, 
    arg: *const c_char,
    mut argv: ...
) -> c_int {
    let mut vec = Vec::new();
    vec.push(arg);

    loop {
        let ptr = argv.arg::<*const c_char>();
        vec.push(ptr);
        if ptr.is_null() {
            break;
        }
    }
    
    execvp(file, vec.as_ptr())
}

#[no_mangle]
unsafe extern "C" fn execv(
    prog: *const c_char, 
    argv: *const *const c_char
) -> c_int {
    execve(prog, argv, environ as *const _)
}

#[no_mangle]
unsafe extern "C" fn execve(
    prog: *const c_char, 
    argv: *const *const c_char, 
    envp: *const *const c_char
) -> c_int {
    let err = rustix::runtime::execve(CStr::from_ptr(prog), argv as *const *const _, envp as *const *const _);

    set_errno(Errno(err.raw_os_error()));
    -1
}

#[no_mangle]
unsafe extern "C" fn execvp(file: *const c_char, argv: *const *const c_char) -> c_int {
    let cwd = rustix::fs::cwd();

    let file = CStr::from_ptr(file);
    if file.to_bytes().contains(&b'/') {
        let err = rustix::runtime::execve(file, argv.cast(), environ.cast());
        set_errno(Errno(err.raw_os_error()));
        return -1;
    }

    let path = _getenv(rustix::cstr!("PATH"));
    let path = if path.is_null() {
        rustix::cstr!("/bin:/usr/bin")
    } else {
        CStr::from_ptr(path)
    };

    let mut access_error = false;
    for dir in path.to_bytes().split(|byte| *byte == b':') {
        // Open the directory and use `execveat` to avoid needing to
        // dynamically allocate a combined path string ourselves.
        let result = rustix::fs::openat(
            cwd,
            dir,
            OFlags::PATH | OFlags::DIRECTORY | OFlags::CLOEXEC | OFlags::NOCTTY,
            Mode::empty(),
        )
        .and_then::<(), _>(|dirfd| {
            let mut error = rustix::runtime::execveat(
                &dirfd,
                file,
                argv.cast(),
                environ.cast(),
                AtFlags::empty(),
            );

            // If `execveat` is unsupported, emulate it with `execve`, without
            // allocating. This trusts /proc/self/fd.
            #[cfg(any(target_os = "android", target_os = "linux"))]
            if let rustix::io::Errno::NOSYS = error {
                let fd = rustix::fs::openat(
                    &dirfd,
                    file,
                    OFlags::PATH | OFlags::CLOEXEC | OFlags::NOCTTY,
                    Mode::empty(),
                )?;
                const PREFIX: &[u8] = b"/proc/self/fd/";
                const PREFIX_LEN: usize = PREFIX.len();
                let mut buf = [0_u8; PREFIX_LEN + 20 + 1];
                buf[..PREFIX_LEN].copy_from_slice(PREFIX);
                let fd_dec = rustix::path::DecInt::from_fd(&fd);
                let fd_bytes = fd_dec.as_c_str().to_bytes_with_nul();
                buf[PREFIX_LEN..PREFIX_LEN + fd_bytes.len()].copy_from_slice(fd_bytes);
                error = rustix::runtime::execve(
                    CStr::from_bytes_with_nul_unchecked(&buf),
                    argv.cast(),
                    environ.cast(),
                );
            }

            Err(error)
        });

        match result {
            Ok(()) => (),
            Err(rustix::io::Errno::ACCESS) => access_error = true,
            Err(rustix::io::Errno::NOENT) | Err(rustix::io::Errno::NOTDIR) => {}
            Err(err) => {
                set_errno(Errno(err.raw_os_error()));
                return -1;
            }
        }
    }

    set_errno(Errno(if access_error {
        libc::EACCES
    } else {
        libc::ENOENT
    }));
    -1
}

#[no_mangle]
unsafe extern "C" fn fexecve(
    fd: c_int, 
    argv: *const *const c_char, 
    envp: *const *const c_char
) -> c_int {
    let err = rustix::runtime::execveat(BorrowedFd::borrow_raw(fd), rustix::cstr!("") ,argv as *const *const _, envp as *const *const _, AtFlags::EMPTY_PATH);

    // TODO: old kernel support

    set_errno(Errno(err.raw_os_error()));
    -1
}