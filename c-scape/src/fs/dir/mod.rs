mod dirfd;
mod opendir;
#[cfg(not(target_os = "wasi"))]
mod readdir;

use rustix::fd::OwnedFd;

union LibcDirStorage {
    dirent: libc::dirent,
    dirent64: libc::dirent64,
}

struct CScapeDir {
    dir: rustix::fs::Dir,
    storage: LibcDirStorage,
    fd: OwnedFd,
}
