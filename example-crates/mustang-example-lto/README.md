This crate demonstrates the use of the `mustang::can_run_this!()` macro in a
"Hello, world" crate.

When compiled for a non-mustang target, it doesn't depend on nightly Rust and
there are no extra dependencies:

```console
$ cargo tree
mustang-example v0.0.0 (…/example-crates/mustang-example)
└── mustang v0.13.2 (…)
```

When compiled for a mustang target, it depends on nightly Rust and it has the
c-gull, c-scape, and origin dependencies:

```console
$ cargo tree --target=x86_64-mustang-linux-gnu
mustang-example v0.0.0 (…/example-crates/mustang-example)
└── mustang v0.14.0 (…)
    ├── c-gull v0.15.0
    │   ├── c-scape v0.15.5
    │   │   ├── errno v0.3.3
    │   │   │   └── libc v0.2.148
    │   │   ├── libc v0.2.148
…
```
