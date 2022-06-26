use origin::*;

fn main() {
    eprintln!("Hello from main");

    at_exit(Box::new(|| eprintln!("Hello from an at_exit handler")));
    at_thread_exit(Box::new(|| {
        eprintln!("Hello from a main-thread at_thread_exit handler")
    }));

    let thread = create_thread(
        Box::new(|| {
            eprintln!("Hello from thread {:?}", current_thread_id());
            at_thread_exit(Box::new(|| {
                eprintln!("Hello from another thread at_thread_exit handler")
            }));
            None
        }),
        2 * 1024 * 1024,
        default_guard_size(),
    )
    .unwrap();

    unsafe {
        join_thread(thread);
    }

    eprintln!("Goodbye from main");
    exit(0);
}
