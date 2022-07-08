<div align="center">
  <h1><code>c-scape</code></h1>

  <p>
    <strong>A libc implementation in Rust</strong>
  </p>

  <p>
    <a href="https://github.com/sunfishcode/mustang/actions?query=workflow%3ACI"><img src="https://github.com/sunfishcode/mustang/workflows/CI/badge.svg" alt="Github Actions CI Status" /></a>
    <a href="https://bytecodealliance.zulipchat.com/#narrow/stream/206238-general"><img src="https://img.shields.io/badge/zulip-join_chat-brightgreen.svg" alt="zulip chat" /></a>
    <a href="https://crates.io/crates/c-scape"><img src="https://img.shields.io/crates/v/c-scape.svg" alt="crates.io page" /></a>
    <a href="https://docs.rs/c-scape"><img src="https://docs.rs/c-scape/badge.svg" alt="docs.rs docs" /></a>
  </p>
</div>

c-scape is a libc implementation. It is an implementation of the ABI described
by the [libc] crate.

It is implemented in terms of crates written in Rust, such as [rustix],
[origin], [sync-resolve], [libm], [realpath-ext], and [memchr].

Currently it only supports `*-*-linux-gnu` ABIs, though other ABIs could be
added in the future. And currently this mostly focused on features needed by
Rust programs, so it doesn't have many C-idiomatic things like `printf` yet, but
they could be added in the future.

The goal is to have very little code in c-scape itself, by factoring out all of
the significant functionality into independent crates with more Rust-idiomatic
APIs, with c-scape just wrapping those APIs to implement the C ABIs.

This is currently experimental, incomplete, and some things aren't optimized.

This is part of the [Mustang] project, building Rust programs written entirely
in Rust.

## Using c-scape in non-mustang programs

c-scape can also be used as a drop-in (partial) libc replacement in non-mustang
builds, provided you're using nightly Rust. To use it, just change your typical
libc dependency in Cargo.toml to this:

```toml
libc = { version = "<c-scape version>", package = "c-scape" }
```

and c-scape will replace as many of the system libc implementation with its own
implementations as it can. In particular, it can't replace `malloc` or any of
the pthread functions in this configuration, but it can replace many other
things.

See the [libc-replacement example] for more details.

[libc-replacement example]: https://github.com/sunfishcode/mustang/blob/main/test-crates/libc-replacement/README.md
[Mustang]: https://github.com/sunfishcode/mustang/
[rustix]: https://crates.io/crates/rustix
[origin]: https://crates.io/crates/origin
[sync-resolve]: https://crates.io/crates/sync-resolve
[libm]: https://crates.io/crates/libm
[libc]: https://crates.io/crates/libc
[realpath-ext]: https://crates.io/crates/realpath-ext
[memchr]: https://crates.io/crates/memchr
[mustang]: https://crates.io/crates/mustang
