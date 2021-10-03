C-ABI-compatible libc, libm, libpthread, and libunwind interfaces, implemented
in terms of crates written in Rust. Currently this only supports `*-*-linux-gnu`
ABIs, though other ABIs could be added.

The goal is to have very little code in `c-scape` itself, by factoring out all
of the significant functionality into independent crates with more
Rust-idiomatic APIs, with `c-scape` just wrapping those APIs to implement the
C ABIs.

This is currently experimental, incomplete, and some things aren't optimized.

c-scape implements `malloc` using the Rust global allocator, and the default
Rust global allocator is implemented using `malloc`, so it's necessary to
enable a different global allocator. The `mustang` crate handles this
automatically.
