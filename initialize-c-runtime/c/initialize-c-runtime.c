//! Before any (hosted) C code can be executed, it's necessary to initialize
//! the C runtime. When enabled, this crate provides a small (freestanding) C
//! function which performs this task.
//!
//! It also registers a cleanup function to flush the `stdio` streams on exit.

// Use only "freestanding" headers here.
#include <stddef.h>

/// This uses `__builtin_setjmp`, which is different than `setjmp` from
/// `<setjmp.h>`. See this page:
///
/// <https://gcc.gnu.org/onlinedocs/gcc/Nonlocal-Gotos.htm>
///
/// for details on `__builtin_setjmp` and declaring `buf`.
///
/// We use `__builtin_setjmp` rather than plain `setjmp` because `setjmp` is
/// not a "freestanding" function. Technically, there is no guarantee that
/// `__builtin_setjmp` is safe to be used either, but it's provided by the
/// compiler itself, and it does less work, so it's the best we can do.
///
/// Declare the jmp buf according to the documentation linked above. Except we
/// use `void *` instead of `intptr_t` for compatibility with clang.
static void *buf[5];

/// A "main" function for `__libc_start_main` to call. We want to call the
/// `real` main ourselves, so this main just does a `longjmp` so we can regain
/// control.
static int catch_main(int argc, char **argv, char **envp) {
  (void)argc;
  (void)argv;
  (void)envp;

  // Jump out through `__libc_start_main`, just before it would call `exit`,
  // back to the `setjmp` in `mustang_initialize_c_runtime`.
  __builtin_longjmp(buf, 1);

  return 0; // Not reached.
}

/// Depending on which C runtime we have and which version, it may either call
/// the `.init_array` functions itself, or expect to be passed a function which
/// does so. Our main requirement is to know what it's going to do, so we
/// pass it a function that does the initialization, so that either way, the
/// initialization gets done, and we can know that we shouldn't do it
/// ourselves.
///
/// We don't do this for the `.fini_array` functions because those are called
/// from `exit`.
static void call_init_array_functions(int argc, char **argv, char **envp) {
  extern void (*__init_array_start[])(int, char **, char **);
  extern void (*__init_array_end[])(int, char **, char **);

  for (void (**p)(int, char **, char **) = __init_array_start;
       p != __init_array_end; ++p) {
    (*p)(argc, argv, envp);
  }
}

/// The `__libc_start_main` function, as specified here:
///
/// <https://refspecs.linuxbase.org/LSB_5.0.0/LSB-Core-generic/LSB-Core-generic/baselib---libc-start-main-.html>
///
/// Except that we support the GLIBC extension of passing `argc`, `argv`, and
/// `envp` to `.init_array` functions.
int __libc_start_main(int (*main)(int, char **, char **), int argc,
                      char **ubp_av, void (*init)(int, char **, char **),
                      void (*fini)(void), void (*rtld_fini)(void),
                      void *stack_end) __attribute__((noreturn));

void mustang_initialize_c_runtime(int argc, char **ubp_av);
void mustang_initialize_c_runtime(int argc, char **ubp_av) {
  if (__builtin_setjmp(buf) == 0) {
    (void)__libc_start_main(catch_main, argc, ubp_av, call_init_array_functions,
                            NULL, NULL,
                            ubp_av // stack_end, or close enough.
    );
  }
}

// For cleanup, we can use "hosted" headers.
#include <stdio.h>

/// The C runtime flushes all open `FILE` streams before exiting.
static void cleanup(void) { (void)fflush(NULL); }

/// Register the `cleanup` function, with priority 0, so that it runs after
/// other cleanups.
__attribute__((used, section(".fini_array.00000"))) static void (
    *register_cleanup)(void) = cleanup;
