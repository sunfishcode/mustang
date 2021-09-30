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

    // test-initialize-c-runtime deliberately link in C runtime symbols.
    if name != "test-initialize-c-runtime" {
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

        let output = Command::new("ldd")
            .arg(&format!(
                "target/{}-mustang-linux-gnu/debug/examples/{}",
                arch, name
            ))
            .output()
            .unwrap();
        assert_eq!(
            "\tstatically linked\n",
            String::from_utf8_lossy(&output.stdout),
            "example {} had unexpected undefined symbols",
            name
        );
    }
}

// TODO: Mustang can't quite compile this yet: the `Command` API needs
// child-process support.
#[cfg_attr(target_vendor = "mustang", ignore)]
#[test]
#[ignore]
fn test() {
    test_example(
        "hello",
        "",
        "Hello, world!\n",
        ".｡oO(Threads spun up by origin! 🧵)\n\
         .｡oO(This process was started by origin! 🎯)\n\
         .｡oO(Environment variables initialized by c-scape! 🌱)\n\
         .｡oO(I/O performed by c-scape using rsix! 🌊)\n\
         .｡oO(This process will be exited by c-scape using rsix! 🚪)\n",
    );
    test_example(
        "test-args",
        "",
        "",
        ".｡oO(Threads spun up by origin! 🧵)\n\
         .｡oO(This process was started by origin! 🎯)\n\
         .｡oO(Environment variables initialized by c-scape! 🌱)\n\
         .｡oO(This process will be exited by c-scape using rsix! 🚪)\n",
    );
    test_example(
        "test-ctor",
        "",
        "",
        ".｡oO(Threads spun up by origin! 🧵)\n\
         .｡oO(This process was started by origin! 🎯)\n\
         .｡oO(Environment variables initialized by c-scape! 🌱)\n\
         .｡oO(This process will be exited by c-scape using rsix! 🚪)\n",
    );
    test_example(
        "test-environ",
        "",
        "",
        ".｡oO(Threads spun up by origin! 🧵)\n\
         .｡oO(This process was started by origin! 🎯)\n\
         .｡oO(Environment variables initialized by c-scape! 🌱)\n\
         .｡oO(This process will be exited by c-scape using rsix! 🚪)\n",
    );
    test_example(
        "test-workdir",
        "",
        "",
        ".｡oO(Threads spun up by origin! 🧵)\n\
         .｡oO(This process was started by origin! 🎯)\n\
         .｡oO(Environment variables initialized by c-scape! 🌱)\n\
         .｡oO(This process will be exited by c-scape using rsix! 🚪)\n",
    );
    test_example(
        "test-simd",
        "",
        "",
        ".｡oO(Threads spun up by origin! 🧵)\n\
         .｡oO(This process was started by origin! 🎯)\n\
         .｡oO(Environment variables initialized by c-scape! 🌱)\n\
         .｡oO(This process will be exited by c-scape using rsix! 🚪)\n",
    );
    test_example(
        "test-tls",
        "",
        "",
        ".｡oO(Threads spun up by origin! 🧵)\n\
         .｡oO(This process was started by origin! 🎯)\n\
         .｡oO(Environment variables initialized by c-scape! 🌱)\n\
         .｡oO(This process will be exited by c-scape using rsix! 🚪)\n",
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
