#[cfg(all(target_os = "linux", target_env = "gnu"))]
mod lsb_abi;

use core::ops::RangeInclusive;
use libc::c_int;

#[no_mangle]
extern "C" fn isalnum(c: c_int) -> c_int {
    (isdigit(c) != 0 || isalpha(c) != 0) as c_int
}

#[no_mangle]
extern "C" fn isalpha(c: c_int) -> c_int {
    (isupper(c) != 0 || islower(c) != 0) as c_int
}

#[no_mangle]
extern "C" fn isascii(c: c_int) -> c_int {
    const RANGE: RangeInclusive<c_int> = 0..=127;
    RANGE.contains(&c) as c_int
}

#[no_mangle]
extern "C" fn isblank(c: c_int) -> c_int {
    (c == ' ' as c_int || c == '\t' as c_int) as c_int
}

#[no_mangle]
extern "C" fn iscntrl(c: c_int) -> c_int {
    const LOWER_RANGE: RangeInclusive<c_int> = 0..=31;
    (LOWER_RANGE.contains(&c) || c == 127) as c_int
}

#[no_mangle]
extern "C" fn isdigit(c: c_int) -> c_int {
    const RANGE: RangeInclusive<c_int> = ('0' as c_int)..=('9' as c_int);
    RANGE.contains(&c) as c_int
}

#[no_mangle]
extern "C" fn isgraph(c: c_int) -> c_int {
    const RANGE: RangeInclusive<c_int> = 33..=126;
    RANGE.contains(&c) as c_int
}

#[no_mangle]
extern "C" fn islower(c: c_int) -> c_int {
    const RANGE: RangeInclusive<c_int> = ('a' as c_int)..=('z' as c_int);
    RANGE.contains(&c) as c_int
}

#[no_mangle]
extern "C" fn isprint(c: c_int) -> c_int {
    (c == 32 || isgraph(c) != 0) as c_int
}

#[no_mangle]
extern "C" fn ispunct(c: c_int) -> c_int {
    (isgraph(c) != 0 && isalnum(c) == 0) as c_int
}

#[no_mangle]
extern "C" fn isspace(c: c_int) -> c_int {
    let is_space = c == ' ' as c_int
            || c == '\t' as c_int
            || c == '\n' as c_int
            || c == '\r' as c_int
            || c == 0x0b
            || c == 0x0c;

    is_space as c_int
}

#[no_mangle]
extern "C" fn isupper(c: c_int) -> c_int {
    const RANGE: RangeInclusive<c_int> = ('A' as c_int)..=('Z' as c_int);
    RANGE.contains(&c) as c_int
}

const ALPHA_CASE_BIT: c_int = 0x20;

#[no_mangle]
extern "C" fn isxdigit(c: c_int) -> c_int {
    const HEX_LOWER_RANGE: RangeInclusive<c_int> = ('a' as c_int)..=('f' as c_int);
    let lowered_unchecked = c | ALPHA_CASE_BIT;
    (isdigit(c) != 0 || HEX_LOWER_RANGE.contains(&lowered_unchecked)) as c_int
}

#[no_mangle]
extern "C" fn tolower(c: c_int) -> c_int {
    if isupper(c) != 0 {
        c | ALPHA_CASE_BIT
    } else {
        c
    }
}

#[no_mangle]
extern "C" fn toupper(c: c_int) -> c_int {
    if islower(c) != 0 {
        c & !ALPHA_CASE_BIT
    } else {
        c
    }
}
