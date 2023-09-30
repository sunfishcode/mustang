//! A very simple example using threads.
//!
//! This is [Rust by Example's threads example].
//!
//! [Rust by Example's threads example]: https://doc.rust-lang.org/rust-by-example/std_misc/threads.html

mustang::can_run_this!();

use std::thread;

const NTHREADS: u32 = 10;

// This is the `main` thread
fn main() {
    // Make a vector to hold the children which are spawned.
    let mut children = vec![];

    for i in 0..NTHREADS {
        // Spin up another thread
        children.push(thread::spawn(move || {
            println!("this is thread number {}", i);
        }));
    }

    for child in children {
        // Wait for the thread to finish. Returns a result.
        let _ = child.join();
    }
}
