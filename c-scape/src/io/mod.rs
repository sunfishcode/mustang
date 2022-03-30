#[cfg(not(target_os = "wasi"))]
mod dup;
mod isatty;
mod pipe;
mod read;
mod write;
