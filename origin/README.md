<div align="center">
  <h1><code>origin</code></h1>

  <p>
    <strong>Program and thread startup and shutdown in Rust</strong>
  </p>

  <p>
    <a href="https://github.com/sunfishcode/mustang/actions?query=workflow%3ACI"><img src="https://github.com/sunfishcode/mustang/workflows/CI/badge.svg" alt="Github Actions CI Status" /></a>
    <a href="https://bytecodealliance.zulipchat.com/#narrow/stream/206238-general"><img src="https://img.shields.io/badge/zulip-join_chat-brightgreen.svg" alt="zulip chat" /></a>
    <a href="https://crates.io/crates/origin"><img src="https://img.shields.io/crates/v/origin.svg" alt="crates.io page" /></a>
    <a href="https://docs.rs/origin"><img src="https://docs.rs/origin/badge.svg" alt="docs.rs docs" /></a>
  </p>
</div>

Origin implements program startup and shutdown, as well as thread startup and
shutdown, for Linux, implemented in Rust.

Program startup and shutdown for Linux is traditionally implemented in crt1.o,
and the libc functions `exit`, `atexit`, and `_exit`. And thread startup and
shutdown are traditionally implemented in libpthread functions
`pthread_create`, `pthread_join`, `pthread_detach`, and so on. Origin provides
its own implementations of this functionality, written in Rust.

For an C-ABI-compatible interface to this functionality, see [c-scape].

This is part of the [Mustang] project, building Rust programs written entirely
in Rust. When compiled for non-mustang targets, this library uses the system
crt and libpthread libraries.

## Using origin in non-mustang programs

Origin can also be used as an orginary library, when compiled in non-mustang
targets, provided you're using nightly Rust. In this configuration, origin
disables its own program startup and thread implementations and lets libc
handle those parts. Its API is then implemented in terms of system libc calls,
including pthread calls.

See the [origin-as-just-a-library example] for more details.

[origin-as-just-a-library example]: ../test-crates/origin-as-just-a-library/README.md
[Mustang]: https://github.com/sunfishcode/mustang/
[c-scape]: https://crates.io/crates/c-scape/
