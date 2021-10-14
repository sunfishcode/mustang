The `aarch64_outline_atomics` directory contains the source that
`aarch64_outline_atomics.s` was generated from, by running:

```
cargo rustc --release --target=aarch64-unknown-linux-gnu -- --emit asm
```

with Rust 1.55.

Rust 1.55 is before rustc switched from using inline LL-SC atomics to making
library calls, so we can use those implementations for the library calls.

This does mean, however, that we aren't yet taking advantage of the new
Arm V8.1a single-instruction atomics.

TODO: Optimize the aarch64 atomics using Arm V8.1a.
