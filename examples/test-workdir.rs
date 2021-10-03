mustang::can_run_this!();

fn main() {
    let _cwd = std::env::current_dir().expect("current directory should exist and be accessible");
    let home = std::env::var_os("HOME").expect("HOME should be present in the environment");
    std::env::set_current_dir(&home).expect("should be able to change to home directory");
    let cwd = std::env::current_dir().expect("home directory should exist and be accessible");
    assert_eq!(
        cwd, home,
        "should be in home directory after set_current_dir"
    );
}
