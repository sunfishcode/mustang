#![feature(backtrace)]
#![feature(backtrace_frames)]
#![feature(portable_simd)]
#![feature(asm)]

extern crate mustang;

use std::os::unix::ffi::OsStrExt;
use std::os::raw::{c_char, c_int};
use std::slice;
use std::ffi::{OsStr, CStr, OsString};
use std::cell::Cell;
use std::thread::{self, LocalKey};
use std::sync::atomic::{Ordering, AtomicBool};

fn main() {
    println!("Hello, world!");

    test_simd();
    test_tls(&TLS);
    test_args();
    test_vars();
    test_backtrace();
    test_ctor();
    test_manual_ctor();
}

fn test_simd() {
    use core_simd::*;
    let mut a = f32x4::splat(2.0);
    unsafe { asm!("# {}", in(reg) &mut a) };
    assert_eq!(a, f32x4::splat(2.0));
    assert_eq!(&a as *const _ as usize & 0xf, 0);
}

thread_local!(static TLS: Cell<i32> = Cell::new(1));

fn test_tls(key: &'static LocalKey<Cell<i32>>) {
    key.with(|f| {
        assert_eq!(f.get(), 1);
        f.set(2);
    });
    let t = thread::spawn(move || {
        key.with(|f| {
            assert_eq!(f.get(), 1);
            f.set(3);
        });
    });
    t.join().unwrap();

    key.with(|f| {
        assert_eq!(f.get(), 2);
    });
}

fn test_args() {
    let mut args = std::env::args_os();
    assert!(!args.next().is_none(), "we should receive an argv[0]");
    assert!(args.next().is_none(), "we aren't expecting any further arguments");
}

fn test_vars() {
    // We expect to be passed some environment variables.
    assert!(!std::env::vars_os().next().is_none(), "the environment shouldn't be empty");
    let _path = std::env::var_os("PATH").expect("PATH should be present in the environment");
}

fn test_backtrace() {
    let backtrace = std::backtrace::Backtrace::force_capture();

    let mut frames = backtrace.frames().iter();

    // There are lots of frames between us and mustang. Skip over them until
    // we get to `lang_start`, which is called by `main`.
    while let Some(frame) = frames.next() {
        if format!("{:?}", frame).contains(" fn: \"std::rt::lang_start\"") {
            break;
        }
    }

    let frame = frames.next().expect("expected `main` next");
    assert!(format!("{:?}", frame).contains(" fn: \"main\""));

    let frame = frames.next().expect("expected `mustang::start_rust` next");
    assert!(format!("{:?}", frame).contains(" fn: \"mustang::start_rust\""));

    // Rust's backtrace adds an empty frame at the end.
    let frame = frames.next().expect("expected an empty frame next");
    assert_eq!(format!("{:?}", frame), "[]");

    assert!(frames.next().is_none());
}

static CTOR_INITIALIZED: AtomicBool = AtomicBool::new(false);

#[ctor::ctor]
fn ctor() {
    CTOR_INITIALIZED.store(true, Ordering::Relaxed);
}

fn test_ctor() {
    assert!(CTOR_INITIALIZED.load(Ordering::Relaxed));
}

static MANUAL_CTOR_INITIALIZED: AtomicBool = AtomicBool::new(false);

#[used]
#[link_section = ".init_array"]
static INIT_ARRAY: unsafe extern "C" fn(c_int, *mut *mut c_char, *mut *mut c_char) = {
    unsafe extern "C" fn function(argc: c_int, argv: *mut *mut c_char, envp: *mut *mut c_char) {
        assert_eq!(argc as usize, std::env::args_os().len());

        assert_eq!(slice::from_raw_parts(argv, argc as usize).iter().map(|arg| OsStr::from_bytes(CStr::from_ptr(*arg).to_bytes()).to_owned()).collect::<Vec<OsString>>(),
                   std::env::args_os().collect::<Vec<OsString>>());
        assert_eq!(*argv.add(argc as usize), std::ptr::null_mut());

        assert_ne!(envp, std::ptr::null_mut());

        let mut ptr = envp;
        let mut num_env = 0;
        loop {
            let env = *ptr;
            if env.is_null() {
                break;
            }

            let bytes = CStr::from_ptr(env).to_bytes();
            let mut parts = bytes.splitn(2, |byte| *byte == b'=');
            assert_eq!(std::env::var_os(OsStr::from_bytes(parts.next().unwrap())).unwrap(),
                       OsStr::from_bytes(parts.next().unwrap()));

            num_env += 1;
            ptr = ptr.add(1);
        }
        assert_eq!(num_env, std::env::vars_os().count());

        MANUAL_CTOR_INITIALIZED.store(true, Ordering::Relaxed);
    }
    function
};

fn test_manual_ctor() {
    assert!(MANUAL_CTOR_INITIALIZED.load(Ordering::Relaxed));
}
