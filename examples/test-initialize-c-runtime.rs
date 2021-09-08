extern crate mustang;

#[cfg(feature = "initialize-c-runtime")]
#[link(name = "hello-c-world")]
extern "C" {
    fn hello_c_world();
}

fn main() {
    #[cfg(feature = "initialize-c-runtime")]
    unsafe {
        hello_c_world();
    }

    #[cfg(not(feature = "initialize-c-runtime"))]
    panic!("The \"initialize-c-runtime\" feature is not enabled.");
}
