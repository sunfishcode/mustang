<div align="center">
  <h1><code>origin</code></h1>

  <p>
    <strong>Program and thread startup and shutdown in Rust</strong>
  </p>

  <p>
    <a href="https://github.com/sunfishcode/mustang/actions?query=workflow%3ACI"><img src="https://github.com/sunfishcode/mustang/workflows/CI/badge.svg" alt="Github Actions CI Status" /></a>
    <a href="https://sunfishcode.zulipchat.com/#narrow/stream/217126-wasmtime"><img src="https://img.shields.io/badge/zulip-join_chat-brightgreen.svg" alt="zulip chat" /></a>
    <a href="https://crates.io/crates/origin"><img src="https://img.shields.io/crates/v/origin.svg" alt="crates.io page" /></a>
    <a href="https://docs.rs/origin"><img src="https://docs.rs/origin/badge.svg" alt="docs.rs docs" /></a>
  </p>
</div>

origin implements program startup and shutdown, as well as thread startup and
shutdown, for Linux, implemented in Rust.

Program startup and shutdown for Linux is traditionally implemented in crt1.o,
and the libc functions `exit`, `atexit`, and `_exit`. And thread startup and
shutdown are traditionally implemented in libpthread functions
`pthread_create`, `pthread_join`, `pthread_detach`, and so on. Origin provides
its own implementations of this functionality, written in Rust.

For an C-ABI-compatible interface to this functionality, see [c-scape].

This is part of the [Mustang] project, building Rust programs written entirely
in Rust.

[Mustang]: https://github.com/sunfishcode/mustang/
[c-scape]: https://crates.io/crates/c-scape/
