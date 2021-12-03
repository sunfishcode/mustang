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

C-ABI-compatible libc, libm, libpthread, and libunwind interfaces, implemented
in terms of crates written in Rust, such as [rustix], [origin], [sync-resolve],
[libm], [realpath-ext], [memchr], and [parking\_lot]. Currently this only
supports `*-*-linux-gnu` ABIs, though other ABIs could be added in the future.
And currently this only supports features needed by Rust programs, though more
support for C programs (eg. `printf`) could also be added in the future.

The goal is to have very little code in c-scape itself, by factoring out all of
the significant functionality into independent crates with more Rust-idiomatic
APIs, with c-scape just wrapping those APIs to implement the C ABIs.

This is currently experimental, incomplete, and some things aren't optimized.

c-scape implements `malloc` using the Rust global allocator, and the default
Rust global allocator is implemented using `malloc`, so it's necessary to
enable a different global allocator. The [mustang] crate handles this
automatically.

This is part of the [Mustang] project, building Rust programs written entirely
in Rust.

[Mustang]: https://github.com/sunfishcode/mustang/
[rustix]: https://crates.io/crates/rustix
[origin]: https://crates.io/crates/origin
[sync-resolve]: https://crates.io/crates/sync-resolve
[libm]: https://crates.io/crates/libm
[realpath-ext]: https://crates.io/crates/realpath-ext
[memchr]: https://crates.io/crates/memchr
[parking\_lot]: https://crates.io/crates/parking_lot
[mustang]: https://crates.io/crates/mustang
