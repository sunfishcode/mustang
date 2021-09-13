This is the main public interface to `mustang`. It automatically
pulls in crates to perform process initialization and termination,
provide selected libc symbols, and registers a global allocator.

To use, add a `mustang` dependency to Cargo.toml, and add this line
to your `main.rs`:

```rust
mustang::can_compile_this!();
```

In `--target=*-mustang-*` builds, this activates `mustang` support.
In all other targets, this has no effect.
