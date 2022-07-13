/// Checks that the given expression evaluates to an `Some(t)`,
/// unpacking the option, or returns `-1` and sets `errno` to
/// `EINVAL`.
macro_rules! some_or_ret_einval {
    ($e:expr) => {
        match $e {
            Some(x) => x,
            None => {
                errno::set_errno(errno::Errno(libc::EINVAL));
                return -1;
            }
        }
    };
}