#![feature(backtrace)]
#![feature(backtrace_frames)]

// `c-scape` doesn't yet support threads and tls, but origin does, so test
// that for now.
#[cfg(target_vendor = "mustang")]
extern crate origin;
#[inline(never)]
#[no_mangle]
#[cold]
extern "C" fn __mustang_c_scape() {}

fn main() {
    let backtrace = std::backtrace::Backtrace::force_capture();
    let mut frames = backtrace.frames().iter();

    // There are lots of frames between us and origin. Skip over them until
    // we get to `main`.
    while let Some(frame) = frames.next() {
        if format!("{:?}", frame).contains(" fn: \"main\"") {
            break;
        }
    }

    let frame = frames.next().expect("expected `origin::rust` after `main`");
    assert!(format!("{:?}", frame).contains(" fn: \"origin::rust\""));

    // Rust's backtrace adds an empty frame at the end.
    let frame = frames.next().expect("expected an empty frame next");
    assert_eq!(format!("{:?}", frame), "[]");

    assert!(frames.next().is_none());
}
