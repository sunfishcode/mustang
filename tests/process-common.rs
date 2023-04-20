//! The following is derived from Rust's
//! library/std/src/sys/unix/process/process_common/tests.rs at revision
//! a52c79e859142c1cd5c0c5bdb73f16b754e1b98f.

mustang::can_run_this!();

use std::ffi::OsStr;
use std::io::{Read, Write};
use std::mem;
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use std::ptr;

#[doc(hidden)]
pub trait IsMinusOne {
    fn is_minus_one(&self) -> bool;
}

macro_rules! impl_is_minus_one {
    ($($t:ident)*) => ($(impl IsMinusOne for $t {
        fn is_minus_one(&self) -> bool {
            *self == -1
        }
    })*)
}

impl_is_minus_one! { i8 i16 i32 i64 isize }

pub fn cvt<T: IsMinusOne>(t: T) -> std::io::Result<T> {
    if t.is_minus_one() {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(t)
    }
}

pub fn cvt_r<T, F>(mut f: F) -> std::io::Result<T>
where
    T: IsMinusOne,
    F: FnMut() -> T,
{
    loop {
        match cvt(f()) {
            Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            other => return other,
        }
    }
}

#[allow(dead_code)] // Not used on all platforms.
pub fn cvt_nz(error: libc::c_int) -> std::io::Result<()> {
    if error == 0 {
        Ok(())
    } else {
        Err(std::io::Error::from_raw_os_error(error))
    }
}

macro_rules! t {
    ($e:expr) => {
        match $e {
            Ok(t) => t,
            Err(e) => panic!("received error for `{}`: {}", stringify!($e), e),
        }
    };
}

#[test]
#[cfg_attr(
    any(
        // See #14232 for more information, but it appears that signal delivery to a
        // newly spawned process may just be raced in the macOS, so to prevent this
        // test from being flaky we ignore it on macOS.
        target_os = "macos",
        // When run under our current QEMU emulation test suite this test fails,
        // although the reason isn't very clear as to why. For now this test is
        // ignored there.
        target_arch = "arm",
        target_arch = "aarch64",
        target_arch = "riscv64",
    ),
    ignore
)]
fn test_process_mask() {
    // Test to make sure that a signal mask *does* get inherited.
    fn test_inner(mut cmd: Command) {
        unsafe {
            let mut set = mem::MaybeUninit::<libc::sigset_t>::uninit();
            let mut old_set = mem::MaybeUninit::<libc::sigset_t>::uninit();
            t!(cvt(libc::sigemptyset(set.as_mut_ptr())));
            t!(cvt(libc::sigaddset(set.as_mut_ptr(), libc::SIGINT)));
            t!(cvt_nz(libc::pthread_sigmask(
                libc::SIG_SETMASK,
                set.as_ptr(),
                old_set.as_mut_ptr()
            )));

            cmd.stdin(Stdio::piped());
            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::null());

            let mut child = t!(cmd.spawn());
            let mut stdin_write = child.stdin.take().unwrap();
            let mut stdout_read = child.stdout.take().unwrap();

            t!(cvt_nz(libc::pthread_sigmask(
                libc::SIG_SETMASK,
                old_set.as_ptr(),
                ptr::null_mut()
            )));

            t!(cvt(libc::kill(child.id() as libc::pid_t, libc::SIGINT)));
            // We need to wait until SIGINT is definitely delivered. The
            // easiest way is to write something to cat, and try to read it
            // back: if SIGINT is unmasked, it'll get delivered when cat is
            // next scheduled.
            let _ = stdin_write.write(b"Hello");
            drop(stdin_write);

            // Exactly 5 bytes should be read.
            let mut buf = [0; 5];
            let ret = t!(stdout_read.read(&mut buf));
            assert_eq!(ret, 5);
            assert_eq!(&buf, b"Hello");

            t!(child.wait());
        }
    }

    // A plain `Command::new` uses the posix_spawn path on many platforms.
    let cmd = Command::new(OsStr::new("cat"));
    test_inner(cmd);

    // Specifying `pre_exec` forces the fork/exec path.
    let mut cmd = Command::new(OsStr::new("cat"));
    unsafe { cmd.pre_exec(Box::new(|| Ok(()))) };
    test_inner(cmd);
}

#[test]
#[cfg_attr(
    any(
        // See test_process_mask
        target_os = "macos",
        target_arch = "arm",
        target_arch = "aarch64",
        target_arch = "riscv64",
    ),
    ignore
)]
fn test_process_group_posix_spawn() {
    unsafe {
        // Spawn a cat subprocess that's just going to hang since there is no I/O.
        let mut cmd = Command::new(OsStr::new("cat"));
        cmd.process_group(0);
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::null());
        let mut child = t!(cmd.spawn());

        // Check that we can kill its process group, which means there *is* one.
        t!(cvt(libc::kill(-(child.id() as libc::pid_t), libc::SIGINT)));

        t!(child.wait());
    }
}

#[test]
#[cfg_attr(
    any(
        // See test_process_mask
        target_os = "macos",
        target_arch = "arm",
        target_arch = "aarch64",
        target_arch = "riscv64",
    ),
    ignore
)]
fn test_process_group_no_posix_spawn() {
    unsafe {
        // Same as above, create hang-y cat. This time, force using the non-posix_spawnp path.
        let mut cmd = Command::new(OsStr::new("cat"));
        cmd.process_group(0);
        cmd.pre_exec(Box::new(|| Ok(()))); // pre_exec forces fork + exec
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::null());
        let mut child = t!(cmd.spawn());

        // Check that we can kill its process group, which means there *is* one.
        t!(cvt(libc::kill(-(child.id() as libc::pid_t), libc::SIGINT)));

        t!(child.wait());
    }
}

/*
#[test]
fn test_program_kind() {
    let vectors = &[
        ("foo", ProgramKind::PathLookup),
        ("foo.out", ProgramKind::PathLookup),
        ("./foo", ProgramKind::Relative),
        ("../foo", ProgramKind::Relative),
        ("dir/foo", ProgramKind::Relative),
        // Note that paths on Unix can't contain / in them, so this is actually the directory "fo\\"
        // followed by the file "o".
        ("fo\\/o", ProgramKind::Relative),
        ("/foo", ProgramKind::Absolute),
        ("/dir/../foo", ProgramKind::Absolute),
    ];

    for (program, expected_kind) in vectors {
        assert_eq!(
            ProgramKind::new(program.as_ref()),
            *expected_kind,
            "actual != expected program kind for input {program}",
        );
    }
}
*/
