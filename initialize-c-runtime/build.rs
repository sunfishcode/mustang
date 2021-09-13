fn main() {
    if std::env::var("CARGO_CFG_TARGET_VENDOR").unwrap() == "mustang" {
        cc::Build::new()
            .flag("-ffreestanding")
            .file("c/initialize-c-runtime.c")
            .compile("initialize-c-runtime");

        cc::Build::new()
            .file("c/hello-c-world.c")
            .compile("hello-c-world");
    }
}
