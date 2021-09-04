#[test]
fn test() {
    use std::process::Command;

    #[cfg(target_arch = "x86_64")]
    let arch = "x86_64";
    #[cfg(target_arch = "aarch64")]
    let arch = "aarch64";
    #[cfg(target_arch = "riscv64")]
    let arch = "riscv64gc";
    #[cfg(target_arch = "x86")]
    let arch = "i686";

    let output = Command::new("cargo")
        .arg("+nightly")
        .arg("run")
        .arg("--quiet")
        .arg("-Z")
        .arg("build-std")
        .arg(&format!(
            "--target=../specs/{}-unknown-linux-mustang.json",
            arch
        ))
        .output()
        .unwrap();

    assert!(output.status.success(), "{:?}", output);

    assert_eq!(output.stdout, b"Hello, world!\n");
    assert_eq!(
        output.stderr,
        b"This process was started by mustang! \xF0\x9F\x90\x8E\n"
    );
}
