fn main() {
    println!("Hello, world! pid={}", unsafe { libc::getpid() });
}
