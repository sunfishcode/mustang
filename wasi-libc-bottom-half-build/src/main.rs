use std::process::Command;

fn main() {
    assert!(
        Command::new("cargo")
            .current_dir("../wasi-libc-bottom-half-staticlib")
            .arg("+nightly")
            .arg("build")
            .arg("--verbose")
            .arg("--target=wasm32-wasi")
            .arg("--no-default-features")
            .status()
            .expect("failed to run wasm-ld")
            .success(),
        "cargo build failed to run"
    );

    assert!(
        Command::new("wasm-ld")
            .arg("--whole-archive")
            .arg("../target/wasm32-wasi/debug/libwasi_libc_bottom_half_staticlib.a")
            .arg("-r")
            .arg("-o")
            .arg("libc-bottom-half.o")
            .status()
            .expect("failed to run wasm-ld")
            .success(),
        "wasm-ld failed to run"
    );
}
