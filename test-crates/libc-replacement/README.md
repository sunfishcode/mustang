This crate demonstrates the use of c-gull as a libc replacement, in
non-mustang builds. In mustang builds, c-gull is the only option :-).

c-gull re-exports libc's API, so it can be used as a drop-in replacement.

This line:

```toml
libc = { path = "../../c-gull", package = "c-gull" }
```

tells cargo to use c-gull in place of libc. In a non-mustang configuration,
it doesn't replace the malloc, pthread, or getauxval functions, but it replaces
everything it can, such as the `getpid` function in the example.
