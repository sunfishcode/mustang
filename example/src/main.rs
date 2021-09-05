#![feature(backtrace)]
#![feature(backtrace_frames)]

extern crate mustang;

use std::cell::Cell;
use std::thread::{self, LocalKey};

thread_local!(static TLS: Cell<i32> = Cell::new(1));

fn main() {
    println!("Hello, world!");

    test_tls(&TLS);
    test_args();
    test_vars();
    test_backtrace();
}

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
