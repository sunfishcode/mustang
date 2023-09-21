This crate demonstrates the use of mustang with no_std. Specifying `-Zbuild-std=core,alloc` lets you skip compiling std as it will not be used:

```console
cargo run --target=x86_64-mustang-linux-gnu -Zbuild-std=core,alloc
```

This line:

```toml
mustang = { path = "../../mustang", default-features = false, features = ["thread", "default-alloc"] }
```

tells cargo to not enable the "std" feature which results in no dependencies on std. c-gull will re-export the no-std version of c-scape. c-scape is still necessary because `dlmalloc` and `unwinding` rely on some libc functionality. `dlmalloc` uses libc for syscalls and needs a pthread implementation for `GlobalAlloc`. `unwinding` uses the libc `dl_iterate_phdr` function, see [comment](https://github.com/sunfishcode/mustang/blob/bf6b53a4c5edd1dec71fa65f468b2c76ff96eb62/mustang/Cargo.toml#L19).

You can use either `#[start]` or replacing the C shim as in the example. See the unstable book for more [details](https://doc.rust-lang.org/unstable-book/language-features/lang-items.html#writing-an-executable-without-stdlib). It may or may not be undefined behavior to call `panic!()` from the function as no catch_unwind has been set, see the start feature [tracking issue](https://github.com/rust-lang/rust/issues/29633) and [following issue](https://github.com/rust-lang/rust/issues/107381).

To use tests, make sure to also compile test and std:

```console
cargo test --target=x86_64-mustang-linux-gnu -Zbuild-std=core,alloc,std,test
```
