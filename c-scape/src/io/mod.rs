#[cfg(not(target_os = "wasi"))]
mod dup;
mod read;
mod write;
mod pipe;