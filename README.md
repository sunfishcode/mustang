<div align="center">
  <h1>Mustang</h1>

  <p>
    <strong>Rust programs written entirely in Rust</strong>
  </p>

  <p>
    <a href="https://github.com/sunfishcode/mustang/actions?query=workflow%3ACI"><img src="https://github.com/sunfishcode/mustang/workflows/CI/badge.svg" alt="Github Actions CI Status" /></a>
    <a href="https://bytecodealliance.zulipchat.com/#narrow/stream/206238-general"><img src="https://img.shields.io/badge/zulip-join_chat-brightgreen.svg" alt="zulip chat" /></a>
    <a href="https://crates.io/crates/mustang"><img src="https://img.shields.io/crates/v/mustang.svg" alt="crates.io page" /></a>
    <a href="https://docs.rs/mustang"><img src="https://docs.rs/mustang/badge.svg" alt="docs.rs docs" /></a>
  </p>
</div>

Mustang is a system for building programs built entirely in Rust, meaning they
do not depend on any part of libc or crt1.o, and do not link in any C code.

Why? For fun! And to exercise some components that have other purposes (such as
[`rustix`], [`origin`], [`c-scape`], and [`c-gull`]) but which happen to also be
part of what's needed to do what Mustang is doing. And in the future, possibly
also for experimenting with new kinds of platform ABIs.

Mustang currently runs on Rust Nightly on Linux on x86-64, x86, aarch64, and
riscv64. It aims to support all Linux versions [supported by Rust], though
at this time it's only tested on relatively recent versions. It's complete
enough to run:

 - [ripgrep](https://github.com/sunfishcode/ripgrep/tree/mustang)
 - [coreutils](https://github.com/sunfishcode/coreutils/tree/mustang),
   including the "unix" feature set
 - [async-std](https://github.com/sunfishcode/tide/tree/mustang)
 - [tokio](https://github.com/sunfishcode/tokio/tree/mustang)
 - [bat](https://github.com/sunfishcode/bat/tree/mustang), including git
   support with libgit2
 - [cargo-watch](https://github.com/sunfishcode/cargo-watch/tree/mustang)

Mustang isn't about making anything safer, for the foreseeable future. The
major libc implementations are extraordinarily well tested and mature. Mustang
for its part is experimental and has lots of `unsafe`.

[`origin`]: https://github.com/sunfishcode/origin/
[Rust-idiomatic]: https://docs.rs/origin/latest/origin/
[`c-scape`]: https://github.com/sunfishcode/c-ward/tree/main/c-scape
[`c-gull`]: https://github.com/sunfishcode/c-ward/tree/main/c-gull
[`mustang`]: https://github.com/sunfishcode/mustang/tree/main/mustang
[supported by Rust]: https://doc.rust-lang.org/nightly/rustc/platform-support.html

## Usage

To use it, first install rust-src, which is needed by `-Z build-std`:

```console
$ rustup component add rust-src --toolchain nightly
```

Then, set the `RUST_TARGET_PATH` environment variable to a path to the
`mustang/target-specs` directory, so that you can name `mustang` targets with
`--target=…`. For example, within a mustang repo:

```console
$ export RUST_TARGET_PATH="$PWD/mustang/target-specs"
```

Then, in your own crate, add a dependency on `mustang`:

```toml
[dependencies]
mustang = "<current version>"
```

And add a `mustang::can_run_this!();` to your top-level module (eg. main.rs).
This does nothing in non-`mustang`-target builds, but in `mustang`-target
builds arranges for `mustang`'s libraries to be linked in.

```rust,no_run
mustang::can_run_this!();
```

Then, compile with Rust nightly, using `-Z build-std` and
`--target=<mustang-target>`. For example:

```console
$ cargo +nightly run --quiet -Z build-std --target=x86_64-mustang-linux-gnu --example hello
Hello, world!
$
```

That's a Rust program built entirely from Rust saying "Hello, world!"!

For more detail, mustang has an `env_logger` feature, which you can enable, and set
`RUST_LOG` to see various pieces of mustang in action:
```console
$ RUST_LOG=trace cargo +nightly run --quiet -Z build-std --target=x86_64-mustang-linux-gnu --example hello --features log,env_logger
[2021-06-28T06:28:31Z TRACE origin::program] Program started
[2021-06-28T06:28:31Z TRACE origin::threads] Main Thread[916066] initialized
[2021-06-28T06:28:31Z TRACE origin::program] Calling `.init_array`-registered function `0x5555558fb480(1, 0x7fffffffdb98, 0x7fffffffdba8)`
[2021-06-28T06:28:31Z TRACE origin::program] Calling `main(1, 0x7fffffffdb98, 0x7fffffffdba8)`
Hello, world!
[2021-06-28T06:28:31Z TRACE origin::program] `main` returned `0`
[2021-06-28T06:28:31Z TRACE origin::program] Program exiting
$
```

A simple way to check for uses of libc functions is to use `nm -u`, since
the above commands are configured to link libc dynamically. If `mustang` has
everything covered, there should be no output:

```console
$ nm -u target/x86_64-mustang-linux-gnu/debug/examples/hello
$
```

## C Runtime interop

To compile C code with a `*-mustang-*` target, you may need to
[tell the `cc` crate which C compiler to use]; for example, for `i686-mustang-linux-gnu`,
set the environment variable `CC_i686-mustang-linux-gnu` to
`i686-linux-gnu-gcc`.

[tell the `cc` crate which C compiler to use]: https://github.com/alexcrichton/cc-rs#external-configuration-via-environment-variables

## panic = "abort"

When using `panic = "abort"` in your Cargo.toml, change the `-Z build-std` to
`-Z build-std=panic_abort,std`. See [here] for background.

[here]: https://github.com/rust-lang/wg-cargo-std-aware/issues/29

## Known Limitations

Known limitations in `mustang` include:

 - Dynamic linking isn't implemented yet.
 - Many libc C functions that aren't typically needed by most Rust programs
   aren't implemented yet.

## Alternatives

[Eyra] uses the same underlying libraries as Mustang, but doesn't use a custom
target, and doesn't need `-Z build-std`.

Using [Origin] directly is another option; there are [examples] showing how
to do this. Using origin directly can produce very small binary sizes; the
`tiny` example can produce a 408-byte executable. The big downside of using
Origin directly is that it doesn't provide a `std` implementation.

[Origin Studio] is a demo of what a `std`-like environment on top of Origin
might look like. It has things like `println!`, but it's also currently lots
of features, such as `File`.

[Eyra]: https://github.com/sunfishcode/eyra#readme
[Origin]: https://github.com/sunfishcode/origin#readme
[examples]: https://github.com/sunfishcode/origin/tree/main/example-crates
[Origin Studio]: https://github.com/sunfishcode/origin-studio#readme

## Background

Mustang is partly inspired by similar functionality in [`steed`], but a few
things are different. cargo's [build-std] is now available, which makes it
much easier to work with custom targets. And Mustang is starting with the
approach of starting by replacing libc interfaces and using `std` as-is,
rather than reimplementing `std`. This is likely to evolve, but whatever we
do, a high-level goal of Mustang is to avoid ever having to reimplement `std`.

Where does `mustang` go from here? Will it support feature X, platform Y, or
use case Z? If `origin` can do program startup in Rust, and [`rustix`] can do
system calls in Rust, what does it all mean?

And could `mustang` eventually support new ABIs that aren't limited to passing
C-style `argc`/`argv`(/`envp`) convention, allowing new kinds of program
argument passing?

Let's find out! Come say hi in the [chat] or an [issue].

## How does one port `mustang` to a new architecture?

 - Port [`rustix`] to the architecture, adding assembly sequences for
   making syscalls.
 - Port [`origin`] to the architecture, adding assembly sequences for
   program and thread primitives.
 - Create a target file in `mustang/target-specs`, by first following
   [these instructions] to generate a specification of a built-in target,
   and then:
     - change `is-builtin` to false
     - change `dynamic-linking` to false
     - add `-nostartfiles` and `-Wl,--undefined=_Unwind_Backtrace` to
       pre-link-args
     - add `"vendor": "mustang"`
   See other targets in the `mustang/target-specs` directory for examples.
 - Compile some of the programs in the `examples` directory, using
   the new target. Try `nm -u` on the binaries to check for undefined
   symbols which need to be implemented.
 - Add the architecture to tests/tests.rs.
 - Add CI testing to .github/workflows/main.yml, by copying what's done
   for other architectures.

## How does one port `mustang` to a new OS?

One probably needs to do similar things as for a new architecture, and also
write a new `origin::rust` implementation to handle the OS's convention for
arguments, environment variables, and initialization functions.

## Using mustang in non-mustang programs

In non-`*-mustang-*` targets, importing this crate and using the
`mustang::can_run_this!()` macro has no effect, does not depend on nightly
Rust, and has no extra dependencies.

See the [mustang-example] example for more details.

## Similar crates

c-scape has some similarities to [relibc], but has a different focus. Relibc is
aiming to be a complete libc replacement, while c-scape is just aiming to cover
the things used by Rust's `std` and popular crates. Some parts of Relibc are
implemented in C, while c-scape is implemented entirely in Rust.

c-scape is also similar to [steed]. See the [Background] for details.

The most significant thing that makes c-scape unique though is its design as a
set of wrappers around Rust crates with Rust interfaces. C ABI compatibility is
useful for getting existing code working, but once things are working, we can
simplify and optimize by changing code to call into the Rust interfaces
directly. This can eliminate many uses of raw pointers and C-style
NUL-terminated strings, so it can be much safer.

Other similar projects are [tiny-std] and [veneer]. Similar to steed, they
contain their own implementations of std.

[relibc]: https://gitlab.redox-os.org/redox-os/relibc/
[steed]: https://github.com/japaric/steed
[tiny-std]: https://github.com/MarcusGrass/tiny-std
[veneer]: https://crates.io/crates/veneer
[`steed`]: https://github.com/japaric/steed
[build-std]: https://doc.rust-lang.org/cargo/reference/unstable.html#build-std
[Rust itself already does this]: https://github.com/rust-lang/rust/blob/6bed1f0bc3cc50c10aab26d5f94b16a00776b8a5/library/std/src/sys/unix/mod.rs#L71
[`rustix`]: https://github.com/bytecodealliance/rustix#readme
[`origin`]: https://github.com/sunfishcode/origin#readme
[chat]: https://bytecodealliance.zulipchat.com/#narrow/stream/206238-general
[issue]: https://github.com/sunfishcode/mustang/issues
[these instructions]: https://docs.rust-embedded.org/embedonomicon/custom-target.html#fill-the-target-file
[Background]: #background
[mustang-example]: https://github.com/sunfishcode/mustang/blob/main/example-crates/mustang-example/README.md
