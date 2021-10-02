//! Run the programs in the `examples` directory and compare their outputs with
//! expected outputs.

mustang::can_compile_this!();

use similar_asserts::assert_eq;

macro_rules! assert_eq_str {
    ($a:expr, $b:expr) => {{
        assert_eq!(String::from_utf8_lossy($a).lines().collect::<Vec<_>>(), String::from_utf8_lossy($b).lines().collect::<Vec<_>>());
        assert_eq!($a, $b);
    }};

    ($a:expr, $b:expr, $($arg:tt)+) => {{
        assert_eq!(String::from_utf8_lossy($a).lines().collect::<Vec<_>>(), String::from_utf8_lossy($b).lines().collect::<Vec<_>>(), $($arg)+);
        assert_eq!($a, $b, $($arg)+);
    }};
}

fn test_example(name: &str, features: &str, stdout: &str, stderr: &str) {
    use std::process::Command;

    #[cfg(target_arch = "x86_64")]
    let arch = "x86_64";
    #[cfg(target_arch = "aarch64")]
    let arch = "aarch64";
    #[cfg(target_arch = "riscv64")]
    let arch = "riscv64gc";
    #[cfg(target_arch = "x86")]
    let arch = "i686";

    let mut command = Command::new("cargo");
    command.arg("+nightly").arg("run").arg("--quiet");
    if !features.is_empty() {
        command
            .arg("--no-default-features")
            .arg("--features")
            .arg(features);
    }
    command
        .arg("-Z")
        .arg("build-std")
        .arg(&format!("--target=specs/{}-mustang-linux-gnu.json", arch))
        .arg("--example")
        .arg(name);
    let output = command.output().unwrap();

    assert_eq_str!(
        stderr.as_bytes(),
        &output.stderr,
        "example {} had unexpected stderr, with {:?}",
        name,
        output
    );
    assert_eq_str!(
        stdout.as_bytes(),
        &output.stdout,
        "example {} had unexpected stdout, with {:?}",
        name,
        output
    );
    assert!(
        output.status.success(),
        "example {} failed with {:?}",
        name,
        output
    );

    let output = Command::new("nm")
        .arg("-u")
        .arg(&format!(
            "target/{}-mustang-linux-gnu/debug/examples/{}",
            arch, name
        ))
        .output()
        .unwrap();
    assert_eq!(
        "",
        String::from_utf8_lossy(&output.stdout),
        "example {} had unexpected undefined symbols",
        name
    );

    let output = Command::new("readelf")
        .arg("-d")
        .arg(&format!(
            "target/{}-mustang-linux-gnu/debug/examples/{}",
            arch, name
        ))
        .output()
        .unwrap();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        assert!(
            !line.contains("NEEDED"),
            "example {} had unexpected dynamic library dependencies: {}",
            name,
            line
        );
    }
}

// TODO: Mustang can't quite compile this yet: the `Command` API needs
// child-process support.
#[cfg_attr(target_vendor = "mustang", ignore)]
#[test]
fn test() {
    test_example("empty", "", "", "");
    test_example("hello", "", "Hello, world!\n", "");
    test_example("test-args", "", "", "");
    test_example("test-ctor", "", "", "");
    test_example("test-environ", "", "", "");
    test_example("test-workdir", "", "", "");
    test_example("test-simd", "", "", "");
    test_example("test-tls", "", "", "");
}
