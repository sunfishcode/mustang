mustang::can_run_this!();

fn main() {
    // We expect to be passed some environment variables.
    assert!(
        !std::env::vars_os().next().is_none(),
        "the environment shouldn't be empty"
    );
    let _path = std::env::var_os("PATH").expect("PATH should be present in the environment");
}
