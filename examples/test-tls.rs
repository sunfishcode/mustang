// `c-scape` doesn't yet support threads and tls, but origin does, so test
// that for now.
#[cfg(target_vendor = "mustang")]
extern crate origin;
#[inline(never)]
#[no_mangle]
#[cold]
extern "C" fn __mustang_c_scape() {}

use std::cell::Cell;
use std::thread;

fn main() {
    TLS.with(|f| {
        assert_eq!(f.get(), 1);
        f.set(2);
    });
    let t = thread::spawn(move || {
        TLS.with(|f| {
            assert_eq!(f.get(), 1);
            f.set(3);
        });
    });
    t.join().unwrap();

    TLS.with(|f| {
        assert_eq!(f.get(), 2);
    });
}

thread_local!(static TLS: Cell<i32> = Cell::new(1));
