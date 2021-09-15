// The following is derived from Rust's
// src/tools/cargo/crates/cargo-test-support/src/lib.rs at revision
// 497ee321af3b8496eaccd7af7b437f18bab81abf.

// Bare metal platforms usually have very small amounts of RAM
// (in the order of hundreds of KB)
pub const DEFAULT_BUF_SIZE: usize = if cfg!(target_os = "espidf") {
    512
} else {
    8 * 1024
};

#[allow(dead_code)] // not used on emscripten
pub mod test {
    use rand::RngCore;
    use std::env;
    use std::fs;
    use std::path::{Path, PathBuf};

    pub struct TempDir(PathBuf);

    impl TempDir {
        pub fn join(&self, path: &str) -> PathBuf {
            let TempDir(ref p) = *self;
            p.join(path)
        }

        pub fn path(&self) -> &Path {
            let TempDir(ref p) = *self;
            p
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            // Gee, seeing how we're testing the fs module I sure hope that we
            // at least implement this correctly!
            let TempDir(ref p) = *self;
            fs::remove_dir_all(p).unwrap();
        }
    }

    pub fn tmpdir() -> TempDir {
        let p = env::temp_dir();
        let mut r = rand::thread_rng();
        let ret = p.join(&format!("rust-{}", r.next_u32()));
        fs::create_dir(&ret).unwrap();
        TempDir(ret)
    }
}
