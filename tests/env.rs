//! The following is derived from Rust's
//! library/std/src/env/tests.rs at revision
//! 497ee321af3b8496eaccd7af7b437f18bab81abf.

mustang::can_compile_this!();

use std::env::*;

use std::path::Path;

#[test]
#[cfg_attr(any(target_os = "emscripten", target_env = "sgx"), ignore)]
fn test_self_exe_path() {
    let path = current_exe();
    assert!(path.is_ok());
    let path = path.unwrap();

    // Hard to test this function
    assert!(path.is_absolute());
}

#[test]
fn test() {
    assert!((!Path::new("test-path").is_absolute()));

    #[cfg(not(target_env = "sgx"))]
    current_dir().unwrap();
}

#[test]
#[cfg(windows)]
fn split_paths_windows() {
    use std::path::PathBuf;

    fn check_parse(unparsed: &str, parsed: &[&str]) -> bool {
        split_paths(unparsed).collect::<Vec<_>>()
            == parsed.iter().map(|s| PathBuf::from(*s)).collect::<Vec<_>>()
    }

    assert!(check_parse("", &mut [""]));
    assert!(check_parse(r#""""#, &mut [""]));
    assert!(check_parse(";;", &mut ["", "", ""]));
    assert!(check_parse(r"c:\", &mut [r"c:\"]));
    assert!(check_parse(r"c:\;", &mut [r"c:\", ""]));
    assert!(check_parse(
        r"c:\;c:\Program Files\",
        &mut [r"c:\", r"c:\Program Files\"]
    ));
    assert!(check_parse(r#"c:\;c:\"foo"\"#, &mut [r"c:\", r"c:\foo\"]));
    assert!(check_parse(
        r#"c:\;c:\"foo;bar"\;c:\baz"#,
        &mut [r"c:\", r"c:\foo;bar\", r"c:\baz"]
    ));
}

#[test]
#[cfg(unix)]
fn split_paths_unix() {
    use std::path::PathBuf;

    fn check_parse(unparsed: &str, parsed: &[&str]) -> bool {
        split_paths(unparsed).collect::<Vec<_>>()
            == parsed.iter().map(|s| PathBuf::from(*s)).collect::<Vec<_>>()
    }

    assert!(check_parse("", &mut [""]));
    assert!(check_parse("::", &mut ["", "", ""]));
    assert!(check_parse("/", &mut ["/"]));
    assert!(check_parse("/:", &mut ["/", ""]));
    assert!(check_parse("/:/usr/local", &mut ["/", "/usr/local"]));
}

#[test]
#[cfg(unix)]
fn join_paths_unix() {
    use std::ffi::OsStr;

    fn test_eq(input: &[&str], output: &str) -> bool {
        &*join_paths(input.iter().cloned()).unwrap() == OsStr::new(output)
    }

    assert!(test_eq(&[], ""));
    assert!(test_eq(
        &["/bin", "/usr/bin", "/usr/local/bin"],
        "/bin:/usr/bin:/usr/local/bin"
    ));
    assert!(test_eq(
        &["", "/bin", "", "", "/usr/bin", ""],
        ":/bin:::/usr/bin:"
    ));
    assert!(join_paths(["/te:st"].iter().cloned()).is_err());
}

#[test]
#[cfg(windows)]
fn join_paths_windows() {
    use std::ffi::OsStr;

    fn test_eq(input: &[&str], output: &str) -> bool {
        &*join_paths(input.iter().cloned()).unwrap() == OsStr::new(output)
    }

    assert!(test_eq(&[], ""));
    assert!(test_eq(&[r"c:\windows", r"c:\"], r"c:\windows;c:\"));
    assert!(test_eq(
        &["", r"c:\windows", "", "", r"c:\", ""],
        r";c:\windows;;;c:\;"
    ));
    assert!(test_eq(&[r"c:\te;st", r"c:\"], r#""c:\te;st";c:\"#));
    assert!(join_paths([r#"c:\te"st"#].iter().cloned()).is_err());
}

#[test]
fn args_debug() {
    assert_eq!(
        format!("Args {{ inner: {:?} }}", args().collect::<Vec<_>>()),
        format!("{:?}", args())
    );
    assert_eq!(
        format!("ArgsOs {{ inner: {:?} }}", args_os().collect::<Vec<_>>()),
        format!("{:?}", args_os())
    );
}
