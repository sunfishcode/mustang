This crate demonstrates the use of c-scape as a libc replacement, in
non-mustang builds. In mustang builds, c-scape is the only option :-).

c-scape re-exports libc's API, so it can be used as a drop-in replacement.

This line:

```toml
libc = { path = "../../c-scape", package = "c-scape" }
```

tells cargo to use c-scape in place of libc. In a non-mustang configuration,
it doesn't replace the malloc, pthread, or getauxval functions, but it replaces
everything it can, such as the `getpid` function in the example.
