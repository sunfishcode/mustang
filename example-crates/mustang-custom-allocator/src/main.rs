mustang::can_run_this!();

#[global_allocator]
static GLOBAL_ALLOCATOR: rustix_dlmalloc::GlobalDlmalloc = rustix_dlmalloc::GlobalDlmalloc;

fn main() {
    println!("Hello, world!");
}
