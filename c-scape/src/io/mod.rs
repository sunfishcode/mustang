#[cfg(not(target_os = "wasi"))]
mod dup;
// #[cfg(any(target_os = "android", target_os = "linux"))]
// mod epoll;
mod isatty;
mod pipe;
mod read;
mod write;
