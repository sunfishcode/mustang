<div align="center">
  <h1><code>c-scape</code></h1>

  <p>
    <strong>A layer underneath c-gull</strong>
  </p>

  <p>
    <a href="https://github.com/sunfishcode/mustang/actions?query=workflow%3ACI"><img src="https://github.com/sunfishcode/mustang/workflows/CI/badge.svg" alt="Github Actions CI Status" /></a>
    <a href="https://bytecodealliance.zulipchat.com/#narrow/stream/206238-general"><img src="https://img.shields.io/badge/zulip-join_chat-brightgreen.svg" alt="zulip chat" /></a>
    <a href="https://crates.io/crates/c-scape"><img src="https://img.shields.io/crates/v/c-scape.svg" alt="crates.io page" /></a>
    <a href="https://docs.rs/c-scape"><img src="https://docs.rs/c-scape/badge.svg" alt="docs.rs docs" /></a>
  </p>
</div>

c-scape is a layer underneath [c-gull]. It provides a subset of libc features,
containing only features that don't require Rust's `std` to implement. This
allows it to be used by `std` itself.

Similar to c-gull, c-scape has "take-charge" and "coexist-with-libc" modes.

[c-gull]: https://crates.io/crates/c-gull
