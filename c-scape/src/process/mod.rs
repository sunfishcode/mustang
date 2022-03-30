mod chdir;
mod egid;
mod euid;
mod gid;
mod groups;
mod kill;
mod pid;
mod sid;
mod uid;
mod wait;

// this entire module is
// #[cfg(not(target_os = "wasi"))]
