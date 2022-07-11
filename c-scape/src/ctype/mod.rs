#[cfg(all(target_os = "linux", target_env = "gnu"))]
mod lsb_abi;

use libc::c_int;

// This is correct because these functions
// expect the value to be representable
// in an unsigned char, or be EOF, or else
// UB occurs.
//
// EOF is -1, which when truncated to u8
// will be 255, correctly returning false for
// all of these functions.
fn truncate_to_u8(c: c_int) -> u8 {
    c as u8
}

#[no_mangle]
unsafe extern "C" fn isalnum(c: c_int) -> c_int {
    libc!(libc::isalnum(c));
    truncate_to_u8(c).is_ascii_alphanumeric() as c_int
}

#[no_mangle]
unsafe extern "C" fn isalpha(c: c_int) -> c_int {
    libc!(libc::isalpha(c));
    truncate_to_u8(c).is_ascii_alphabetic() as c_int
}

#[no_mangle]
unsafe extern "C" fn isblank(c: c_int) -> c_int {
    libc!(libc::isblank(c));
    (c == ' ' as c_int || c == '\t' as c_int) as c_int
}

#[no_mangle]
unsafe extern "C" fn iscntrl(c: c_int) -> c_int {
    libc!(libc::iscntrl(c));
    truncate_to_u8(c).is_ascii_control() as c_int
}

#[no_mangle]
unsafe extern "C" fn isdigit(c: c_int) -> c_int {
    libc!(libc::isdigit(c));
    truncate_to_u8(c).is_ascii_digit() as c_int
}

#[no_mangle]
unsafe extern "C" fn isgraph(c: c_int) -> c_int {
    libc!(libc::isgraph(c));
    truncate_to_u8(c).is_ascii_graphic() as c_int
}

#[no_mangle]
unsafe extern "C" fn islower(c: c_int) -> c_int {
    libc!(libc::islower(c));
    truncate_to_u8(c).is_ascii_lowercase() as c_int
}

#[no_mangle]
unsafe extern "C" fn isprint(c: c_int) -> c_int {
    libc!(libc::isprint(c));
    (c == 32 || isgraph(c) != 0) as c_int
}

#[no_mangle]
unsafe extern "C" fn ispunct(c: c_int) -> c_int {
    libc!(libc::ispunct(c));
    truncate_to_u8(c).is_ascii_punctuation() as c_int
}

#[no_mangle]
unsafe extern "C" fn isspace(c: c_int) -> c_int {
    libc!(libc::isspace(c));

    // We can't use the core function
    // for space testing here because we 
    // have a slightly different defintion.
    let is_space = c == ' ' as c_int
        || c == '\t' as c_int
        || c == '\n' as c_int
        || c == '\r' as c_int
        || c == 0x0b
        || c == 0x0c;

    is_space as c_int
}

#[no_mangle]
unsafe extern "C" fn isupper(c: c_int) -> c_int {
    libc!(libc::isupper(c));
    truncate_to_u8(c).is_ascii_uppercase() as c_int
}

#[no_mangle]
unsafe extern "C" fn isxdigit(c: c_int) -> c_int {
    libc!(libc::isxdigit(c));
    truncate_to_u8(c).is_ascii_hexdigit() as c_int
}

#[no_mangle]
unsafe extern "C" fn tolower(c: c_int) -> c_int {
    libc!(libc::tolower(c));
    truncate_to_u8(c).to_ascii_lowercase() as c_int
}

#[no_mangle]
unsafe extern "C" fn toupper(c: c_int) -> c_int {
    libc!(libc::toupper(c));
    truncate_to_u8(c).to_ascii_uppercase() as c_int
}
