mod dirfd;
mod opendir;
#[cfg(not(target_os = "wasi"))]
mod readdir;

use rustix::fd::OwnedFd;

struct MustangDir {
    dir: rustix::fs::Dir,
    dirent: libc::dirent64,
    fd: OwnedFd,
}