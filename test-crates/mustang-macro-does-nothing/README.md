This crate demonstrates the use of the `mustang::can_run_this!()` macro in a
crate. When compiled for a non-mustang target, it doesn't depend on nightly
Rust and there are no extra dependencies:

```
$ cargo tree
mustang-macro-does-nothing v0.0.0 (.../test-crates/mustang-macro-does-nothing)
└── mustang v0.13.2 (...)
```

When compiled for a mustang target, it depends on nightly Rust and it has the
c-gull and origin dependencies:

```
$ cargo tree --target=x86_64-mustang-linux-gnu
mustang-macro-does-nothing v0.0.0 (.../test-crates/mustang-macro-does-nothing)
└── mustang v0.13.2 (...)
    ├── c-gull v0.14.5
    │   ├── c-scape v0.14.5
    │   │   ├── errno v0.3.3
    │   │   │   └── libc v0.2.148
    │   │   ├── libc v0.2.148
...
```
