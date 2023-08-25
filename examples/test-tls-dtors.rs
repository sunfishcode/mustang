mustang::can_run_this!();

use std::cell::RefCell;
use std::thread;

fn main() {
    TLS.with_borrow_mut(|f| {
        assert_eq!(f.0, 1);
        *f = Thing(2);
    });
    let t = thread::spawn(move || {
        TLS.with_borrow_mut(|f| {
            assert_eq!(f.0, 1);
            *f = Thing(3);
        });
    });
    t.join().unwrap();

    TLS.with_borrow_mut(|f| {
        assert_eq!(f.0, 2);
    });
    eprintln!("main exiting");
}

struct Thing(i32);

impl Drop for Thing {
    fn drop(&mut self) {
        eprintln!("Thing being dropped!");
    }
}

thread_local!(static TLS: RefCell<Thing> = RefCell::new(Thing(1)));
