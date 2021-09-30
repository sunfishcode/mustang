mustang::can_compile_this!();

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
