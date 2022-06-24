//! The following is derived from Rust's
//! library/std/src/sync/barrier.rs at revision
//! 497ee321af3b8496eaccd7af7b437f18bab81abf.

mustang::can_run_this!();

use std::sync::mpsc::{channel, TryRecvError};
use std::sync::{Arc, Barrier};
use std::thread;

#[test]
#[cfg_attr(target_os = "emscripten", ignore)]
fn test_barrier() {
    const N: usize = 10;

    let barrier = Arc::new(Barrier::new(N));
    let (tx, rx) = channel();

    for _ in 0..N - 1 {
        let c = barrier.clone();
        let tx = tx.clone();
        thread::spawn(move || {
            tx.send(c.wait().is_leader()).unwrap();
        });
    }

    // At this point, all spawned threads should be blocked,
    // so we shouldn't get anything from the port
    assert!(matches!(rx.try_recv(), Err(TryRecvError::Empty)));

    let mut leader_found = barrier.wait().is_leader();

    // Now, the barrier is cleared and we should get data.
    for _ in 0..N - 1 {
        if rx.recv().unwrap() {
            assert!(!leader_found);
            leader_found = true;
        }
    }
    assert!(leader_found);
}
