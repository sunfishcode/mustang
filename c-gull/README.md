<div align="center">
  <h1><code>c-gull</code></h1>

  <p>
    <strong>A libc implementation in Rust</strong>
  </p>

  <p>
    <a href="https://github.com/sunfishcode/mustang/actions?query=workflow%3ACI"><img src="https://github.com/sunfishcode/mustang/workflows/CI/badge.svg" alt="Github Actions CI Status" /></a>
    <a href="https://bytecodealliance.zulipchat.com/#narrow/stream/206238-general"><img src="https://img.shields.io/badge/zulip-join_chat-brightgreen.svg" alt="zulip chat" /></a>
    <a href="https://crates.io/crates/c-gull"><img src="https://img.shields.io/crates/v/c-gull.svg" alt="crates.io page" /></a>
    <a href="https://docs.rs/c-gull"><img src="https://docs.rs/c-gull/badge.svg" alt="docs.rs docs" /></a>
  </p>
</div>

c-gull is a libc implementation. It is an implementation of the ABI described
by the [libc] crate.

It is implemented in terms of crates written in Rust, such as [c-scape],
[rustix], [origin], [sync-resolve], [libm], [realpath-ext], [tz-rs],
[printf-compat], and [rand].

Currently it only supports `*-*-linux-gnu` ABIs, though other ABIs could be
added in the future. And currently this mostly focused on features needed by
Rust programs, so it doesn't have many C-idiomatic things like `printf` yet, but
they could be added in the future.

The goal is to have very little code in c-gull itself, by factoring out all of
the significant functionality into independent crates with more Rust-idiomatic
APIs, with c-gull just wrapping those APIs to implement the C ABIs.

This is currently highly experimental, incomplete, and some things aren't
optimized.

This is part of the [Mustang] project, building Rust programs written entirely
in Rust.

## Using c-gull in non-mustang programs

c-gull can also be used as a drop-in (partial) libc replacement in non-mustang
builds, provided you're using nightly Rust. To use it, just change your typical
libc dependency in Cargo.toml to this:

```toml
libc = { version = "<c-gull version>", package = "c-gull" }
```

and c-gull will replace as many of the system libc implementation with its own
implementations as it can. In particular, it can't replace `malloc` or any of
the pthread functions in this configuration, but it can replace many other
things.

See the [libc-replacement example] for more details.

[libc-replacement example]: https://github.com/sunfishcode/mustang/blob/main/test-crates/libc-replacement/README.md
[Mustang]: https://github.com/sunfishcode/mustang/
[c-scape]: https://crates.io/crates/c-scape
[rustix]: https://crates.io/crates/rustix
[origin]: https://crates.io/crates/origin
[sync-resolve]: https://crates.io/crates/sync-resolve
[libm]: https://crates.io/crates/libm
[libc]: https://crates.io/crates/libc
[realpath-ext]: https://crates.io/crates/realpath-ext
[mustang]: https://crates.io/crates/mustang
[tz-rs]: https://crates.io/crates/tz-rs
[printf-compat]: https://crates.io/crates/printf-compat
[rand]: https://crates.io/crates/rand
