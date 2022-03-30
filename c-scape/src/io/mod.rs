#[cfg(not(target_os = "wasi"))]
mod dup;
mod isatty;
mod read;
mod write;
mod pipe;