<div align="center">
  <h1><code>mustang</code></h1>

  <p>
    <strong>Selected C-ABI-compatible libc, libm, and libpthread interfaces</strong>
  </p>

  <p>
    <a href="https://github.com/sunfishcode/mustang/actions?query=workflow%3ACI"><img src="https://github.com/sunfishcode/mustang/workflows/CI/badge.svg" alt="Github Actions CI Status" /></a>
    <a href="https://bytecodealliance.zulipchat.com/#narrow/stream/206238-general"><img src="https://img.shields.io/badge/zulip-join_chat-brightgreen.svg" alt="zulip chat" /></a>
    <a href="https://crates.io/crates/mustang"><img src="https://img.shields.io/crates/v/mustang.svg" alt="crates.io page" /></a>
    <a href="https://docs.rs/mustang"><img src="https://docs.rs/mustang/badge.svg" alt="docs.rs docs" /></a>
  </p>
</div>

Support crate for `*-mustang-*` targets. See [here] for usage instructions.

In non-`*-mustang-*` targets, importing this crate has no effect.

In `*-mustang-*` targets, this crate automatically pulls in crates to perform
program initialization and termination, provide selected libc symbols, and
registers a global allocator. By default, it uses [dlmalloc]; alternatively,
[wee\_alloc] can be enabled with a cargo feature.

To use, add a `mustang` dependency to Cargo.toml, and add this line to your
`main.rs`:

```rust
mustang::can_run_this!();
```

This macro expands to nothing in non-`*-mustang-*` builds.

This is part of the [Mustang] project, building Rust programs written entirely
in Rust.

[Mustang]: https://github.com/sunfishcode/mustang/
[here]: https://github.com/sunfishcode/mustang#usage
[dlmalloc]: https://crates.io/crates/dlmalloc
[wee\_alloc]: https://crates.io/crates/wee_alloc
