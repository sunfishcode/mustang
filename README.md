<div align="center">
  <h1><code>mustang</code></h1>

  <p>
    <strong>Program startup written in Rust</strong>
  </p>

  <p>
    <a href="https://github.com/sunfishcode/mustang/actions?query=workflow%3ACI"><img src="https://github.com/sunfishcode/mustang/workflows/CI/badge.svg" alt="Github Actions CI Status" /></a>
    <a href="https://bytecodealliance.zulipchat.com/#narrow/stream/217126-wasmtime"><img src="https://img.shields.io/badge/zulip-join_chat-brightgreen.svg" alt="zulip chat" /></a>
    <a href="https://crates.io/crates/mustang"><img src="https://img.shields.io/crates/v/mustang.svg" alt="crates.io page" /></a>
    <a href="https://docs.rs/mustang"><img src="https://docs.rs/mustang/badge.svg" alt="docs.rs docs" /></a>
  </p>
</div>

Mustang is a program startup library, similar to `crt1.o`, written entirely
in Rust (nightly).

It's currently experimental, and currently works on Linux, on x86-64, x86,
aarch64, and riscv64gc.

To use it, you first install rust-src, which is needed to use build-std:

```
$ rustup component add rust-src --toolchain nightly
```

Then, set the `RUST_TARGET_PATH` environment variable to a path to mustang's
`specs` directory. For example, within a mustang repo:

```
$ export RUST_TARGET_PATH="$PWD/specs"
```

Then, in your own crate, add a dependency on `mustang`:

```toml
[dependencies]
mustang = { git = "https://github.com/sunfishcode/mustang" }
```

And add an `extern crate` declaration to your main.rs:

```rust
extern crate mustang;
```

Then, compile with Rust nightly, using `-Z build-std` and
`--target=x86_64-unknown-linux-mustang`, for example:

```
$ cargo +nightly run -Z build-std --target=x86_64-unknown-linux-mustang
This process was started by mustang! 🐎
Hello, world!
```

The first line of output is printed by `mustang` in debug builds, to confirm
you have everything set up properly.

## Known Limitations

Known limitations in `mustang` include:

 - The ELF `init` function is not supported, however the more modern
   `.init_array` mechanism is supported.
 - The stdio descriptors are not sanitized, however for Rust programs,
   [Rust itself already does this].
 - No support for stack smashing protection (ssp) yet.
 - No support for dynamic linking yet.

## Background

Mustang is partly inspired by similar functionality in [`steed`]. Mustang is
able to take advantage of [build-std] support, which is now built into cargo,
so it doesn't have to re-implement all of `std`. It can just use `std`.

Where does `mustang` go from here? Will it support feature X, platform Y, or
use case Z? If `mustang` can do program startup in Rust, and [`rsix`] can do
system calls in Rust, what does it all mean? And will `mustang` always print
that silly horse?

And could `mustang` eventually support new ABIs that aren't limited to passing
C-style `argc`/`argv`(/`envp`) convention, allowing new kinds of program
argument passing?

Let's find out! Come say hi in the [chat] or an [issue].

## How does one port `mustang` to a new architecture?

 - Add assembly code to the `_start` function in src/lib.rs to call
   `start_rust`.
 - Create a target file in `specs/`, by first following
   [these instructions] to generate a default target file, and then:
     - change `dynamic-linking` to false
     - add `-nostartfiles` and `-Wl,--require-defined_start` to pre-link-args
   See other targets in the `specs/` directory for examples.
 - Add the architecture to example/test/test.rs.
 - Add CI testing to .github/workflows/main.yml, by copying what's done
   for other architectures.

## How does one port `mustang` to a new OS?

One probably needs to do similar things as for a new architecture, and also
write a new `start_rust` implementation to handle the OS's convention for
arguments, environment variables, and initialization functions.

[`steed`]: https://github.com/japaric/steed
[build-std]: https://doc.rust-lang.org/cargo/reference/unstable.html#build-std
[Rust itself already does this]: https://github.com/rust-lang/rust/blob/6bed1f0bc3cc50c10aab26d5f94b16a00776b8a5/library/std/src/sys/unix/mod.rs#L71
[`rsix`]: https://github.com/bytecodealliance/rsix
[chat]: https://bytecodealliance.zulipchat.com/#narrow/stream/217126-wasmtime
[issue]: https://github.com/sunfishcode/mustang/issues
[these instructions]: https://docs.rust-embedded.org/embedonomicon/custom-target.html#fill-the-target-file
