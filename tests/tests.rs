#[allow(unused_imports)]
use pretty_assertions::{assert_eq, assert_ne};

macro_rules! assert_eq_str {
    ($a:expr, $b:expr) => {{
        assert_eq!(String::from_utf8_lossy($a), String::from_utf8_lossy($b));
        assert_eq!($a, $b);
    }};

    ($a:expr, $b:expr, $($arg:tt)+) => {{
        assert_eq!(String::from_utf8_lossy($a), String::from_utf8_lossy($b), $($arg)+);
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

    let output = Command::new("cargo")
        .arg("+nightly")
        .arg("run")
        .arg("--quiet")
        .arg("--features")
        .arg(features)
        .arg("-Z")
        .arg("build-std")
        .arg(&format!("--target=specs/{}-mustang-linux-gnu.json", arch))
        .arg("--example")
        .arg(name)
        .output()
        .unwrap();

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

    // test-backtrace and test-tls are not fully supported by mustang yet.
    // test-initialize-c-runtime deliberately links in C runtime symbols.
    if name != "test-backtrace" && name != "test-tls" && name != "test-initialize-c-runtime" {
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
    }
}

#[test]
fn test() {
    test_example(
        "hello",
        "",
        "Hello, world!\n",
        ".｡oO(This process was started by origin! 🎯)\n\
         .｡oO(Environment variables initialized by c-scape! 🌱)\n\
         .｡oO(I/O performed by c-scape using rsix! 🌊)\n\
         .｡oO(This process will be exited by c-scape using rsix! 🚪)\n",
    );
    test_example(
        "test-args",
        "",
        "",
        ".｡oO(This process was started by origin! 🎯)\n\
         .｡oO(Environment variables initialized by c-scape! 🌱)\n\
         .｡oO(This process will be exited by c-scape using rsix! 🚪)\n",
    );
    test_example(
        "test-backtrace",
        "",
        "",
        ".｡oO(This process was started by origin! 🎯)\n",
    );
    test_example(
        "test-ctor",
        "",
        "",
        ".｡oO(This process was started by origin! 🎯)\n\
         .｡oO(Environment variables initialized by c-scape! 🌱)\n\
         .｡oO(This process will be exited by c-scape using rsix! 🚪)\n",
    );
    test_example(
        "test-environ",
        "",
        "",
        ".｡oO(This process was started by origin! 🎯)\n\
         .｡oO(Environment variables initialized by c-scape! 🌱)\n\
         .｡oO(This process will be exited by c-scape using rsix! 🚪)\n",
    );
    test_example(
        "test-simd",
        "",
        "",
        ".｡oO(This process was started by origin! 🎯)\n\
         .｡oO(Environment variables initialized by c-scape! 🌱)\n\
         .｡oO(This process will be exited by c-scape using rsix! 🚪)\n",
    );
    test_example(
        "test-tls",
        "",
        "",
        ".｡oO(This process was started by origin! 🎯)\n",
    );
    test_example(
        "test-initialize-c-runtime",
        "initialize-c-runtime",
        "Hello from C!\n",
        ".｡oO(This process was started by origin! 🎯)\n\
         .｡oO(C runtime initialization called by origin! ℂ)\n\
         .｡oO(Environment variables initialized by c-scape! 🌱)\n\
         .｡oO(This process will be exited by c-scape using rsix! 🚪)\n",
    );
}
