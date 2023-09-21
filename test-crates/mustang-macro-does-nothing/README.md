This crate demonstrates the use of the `mustang::can_run_this!()` macro in a
crate. When compiled for a non-mustang target, it doesn't depend on nightly
Rust and there are no extra dependencies:

```
$ cargo tree
mustang-macro-does-nothing v0.0.0 (test-crates/mustang-macro-does-nothing)
└── mustang v0.4.1 (mustang/mustang)
```

When compiled for a mustang target, it depends on nightly Rust and it has the
c-gull and origin dependencies:

```
$ cargo tree --target=x86_64-mustang-linux-gnu
mustang-macro-does-nothing v0.0.0 (test-crates/mustang-macro-does-nothing)
└── mustang v0.11.0 (mustang/mustang)
    ├── c-gull v0.11.0 (c-ward/c-gull)
    │   ├── c-scape v0.11.0 (c-ward/c-scape)
    │   │   ├── errno v0.3.3
    │   │   │   └── libc v0.2.147
    │   │   ├── libc v0.2.147
...
```
