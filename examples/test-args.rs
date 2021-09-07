extern crate mustang;

fn main() {
    let mut args = std::env::args_os();
    assert!(!args.next().is_none(), "we should receive an argv[0]");
    assert!(
        args.next().is_none(),
        "we aren't expecting any further arguments"
    );
}
