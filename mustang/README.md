Support crate for `*-mustang-*` targets.

In non-`*-mustang-*` targets, importing this crate has no effect.

In `*-mustang-*` targets, this crate automatically pulls in crates to perform
program initialization and termination, provide selected libc symbols, and
registers a global allocator.

To use, add a `mustang` dependency to Cargo.toml, and add this line to your
`main.rs`:

```rust
mustang::can_run_this!();
```

This macro expands to nothing in non-`*-mustang-*` builds.
