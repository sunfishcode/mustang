mod dirfd;
mod opendir;
#[cfg(not(target_os = "wasi"))]
mod readdir;

use rustix::fd::OwnedFd;

struct CScapeDir {
    dir: rustix::fs::Dir,
    dirent: libc::dirent64,
    fd: OwnedFd,
}
