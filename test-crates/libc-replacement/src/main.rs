fn main() {
    println!("Hello, world! pid={}", unsafe { libc::getpid() });
    unsafe { libc::printf("Hello world with printf! gid=%u\n\0".as_ptr().cast(), libc::getuid()); }
}
