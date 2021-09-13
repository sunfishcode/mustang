<div align="center">
  <h1><code>mustang</code></h1>

  <p>
    <strong>Programs written entirely in Rust</strong>
  </p>

  <p>
    <a href="https://github.com/sunfishcode/mustang/actions?query=workflow%3ACI"><img src="https://github.com/sunfishcode/mustang/workflows/CI/badge.svg" alt="Github Actions CI Status" /></a>
    <a href="https://bytecodealliance.zulipchat.com/#narrow/stream/217126-wasmtime"><img src="https://img.shields.io/badge/zulip-join_chat-brightgreen.svg" alt="zulip chat" /></a>
    <a href="https://crates.io/crates/mustang"><img src="https://img.shields.io/crates/v/mustang.svg" alt="crates.io page" /></a>
    <a href="https://docs.rs/mustang"><img src="https://docs.rs/mustang/badge.svg" alt="docs.rs docs" /></a>
  </p>
</div>

Mustang is a system for building programs built entirely in Rust, meaning they
do not depend on any part of libc or crt1.o, and do not link in any C code.

Why? For fun! And to exercise some components built for other purposes (such as
[`rsix`]) but which happen to also be part of what's needed to do what Mustang
is doing. And in the future, possibly also for experimenting with new kinds of
platform ABIs and new forms of process argument passing.

Mustang isn't about making anything safer, for the foreseeable future. The
major libc implementations are extraordinarily well tested and mature. Mustang
for its part is experimental and has lots of `unsafe`.

This also isn't about building a complete libc. It currently includes some
things with libc-compatible interfaces, just enough to allow it to slide in
underneath `std`, however even this may not always be necessary. We'll see.

Mustang currently runs on Rust Nightly on Linux on x86-64, aarch64, and x86.

## Usage

To use it, first install rust-src, which is needed by `-Z build-std`:

```
$ rustup component add rust-src --toolchain nightly
```

Then, set the `RUST_TARGET_PATH` environment variable to a path to mustang's
`specs` directory, so that you can name `mustang` targets with `--target=...`.
For example, within a mustang repo:

```
$ export RUST_TARGET_PATH="$PWD/specs"
```

Then, in your own crate, add a dependency on `mustang`:

```toml
[dependencies]
mustang = { git = "https://github.com/sunfishcode/mustang" }
```

And add a `mustang::can_compile_this!();` to your top-level module (eg. main.rs). This
does nothing in non-`mustang`-target builds, but in `mustang`-target builds
arranges for `mustang`'s libraries to be linked in.

```rust
mustang::can_compile_this!();
```

Then, compile with Rust nightly, using `-Z build-std` and
`--target=<mustang-target>`. For example:

```
$ cargo +nightly run --quiet -Z build-std --target=x86_64-mustang-linux-gnu --example hello
.｡oO(This process was started by origin! 🎯)
.｡oO(Environment variables initialized by c-scape! 🌱)
.｡oO(I/O performed by c-scape using rsix! 🌊)
Hello, world!
.｡oO(This process will be exited by c-scape using rsix! 🚪)
$
```

That's a Rust program built entirely from Rust saying "Hello, world!"!

Those `.｡oO` lines are just debugging output to confirm everything is set up
properly. Once `mustang` is more stable, we'll stop printing them.

A simple way to check for uses of libc functions is to use `nm -u`, since
the above commands are configured to link libc dynamically. If `mustang` has
everything covered, there should be no output:

```
$ nm -u target/x86_64-mustang-linux-gnu/debug/examples/hello
$
```

## C Runtime interop

C has a runtime, and if you wish to link with any C libraries, the C runtime
needs to be initialized. `mustang` doesn't do this by default, but it does
support this when the cargo feature "initialize-c-runtime" is enabled.

To compile C code with a `*-mustang-*` target, you may need to
[tell the `cc` crate which C compiler to use]; for example, for `i686-mustang-linux-gnu`,
set the environment variable `CC_i686-mustang-linux-gnu` to
`i686-linux-gnu-gcc`.

[tell the `cc` crate which C compiler to use]: https://github.com/alexcrichton/cc-rs#external-configuration-via-environment-variables

## Known Limitations

Known limitations in `mustang` include:

 - Networking, `current_dir`, spawning new processes, threads, and unwinding
   panics are not implemented yet.
 - No support for dynamic linking yet.

## Background

Mustang is partly inspired by similar functionality in [`steed`], but a few
things are different. cargo's [build-std] is now available, which makes it
much easier to work with custom targets. And Mustang is starting with the
approach of starting by replacing libc interfaces and using `std` as-is,
rather than reimplementing `std`. This is likely to evolve, but whatever we
do, a high-level goal of Mustang is to avoid ever having to reimplement `std`.

Where does `mustang` go from here? Will it support feature X, platform Y, or
use case Z? If `origin` can do program startup in Rust, and [`rsix`] can do
system calls in Rust, what does it all mean?

And could `mustang` eventually support new ABIs that aren't limited to passing
C-style `argc`/`argv`(/`envp`) convention, allowing new kinds of program
argument passing?

Let's find out! Come say hi in the [chat] or an [issue].

## How does one port `mustang` to a new architecture?

 - Port [`rsix`] to the architecture, adding assembly sequences for
   making syscalls on the architecture.
 - Add assembly code to the `_start` function in src/lib.rs to call
   `origin::rust`.
 - Create a target file in `specs/`, by first following
   [these instructions] to generate a default target file, and then:
     - change `dynamic-linking` to false
     - add `-nostdlib`, `-Wl,--require-defined=start`, and
       `-Wl,--require-defined=environ` to pre-link-args
     - add `"vendor": "mustang"`
   See other targets in the `specs/` directory for examples.
 - Add the architecture to example/test/test.rs.
 - Add CI testing to .github/workflows/main.yml, by copying what's done
   for other architectures.

## How does one port `mustang` to a new OS?

One probably needs to do similar things as for a new architecture, and also
write a new `origin::rust` implementation to handle the OS's convention for
arguments, environment variables, and initialization functions.

[`steed`]: https://github.com/japaric/steed
[build-std]: https://doc.rust-lang.org/cargo/reference/unstable.html#build-std
[Rust itself already does this]: https://github.com/rust-lang/rust/blob/6bed1f0bc3cc50c10aab26d5f94b16a00776b8a5/library/std/src/sys/unix/mod.rs#L71
[`rsix`]: https://github.com/bytecodealliance/rsix
[chat]: https://bytecodealliance.zulipchat.com/#narrow/stream/217126-wasmtime
[issue]: https://github.com/sunfishcode/mustang/issues
[these instructions]: https://docs.rust-embedded.org/embedonomicon/custom-target.html#fill-the-target-file
