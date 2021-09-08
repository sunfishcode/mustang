fn main() {
    cc::Build::new()
        .flag("-ffreestanding")
        .file("c/initialize-c-runtime.c")
        .compile("initialize-c-runtime");

    cc::Build::new()
        .file("c/hello-c-world.c")
        .compile("hello-c-world");
}
