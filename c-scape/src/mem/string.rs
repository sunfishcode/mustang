use core::cell::SyncUnsafeCell;
use core::ptr;
use libc::{c_char, c_int, c_schar, malloc, memcpy};

const NUL: c_char = 0;

#[no_mangle]
unsafe extern "C" fn stpcpy(mut d: *mut c_char, mut s: *const c_char) -> *mut c_char {
    libc!(libc::stpcpy(d, s));

    loop {
        *d = *s;

        if *d == NUL {
            break;
        }

        d = d.add(1);
        s = s.add(1);
    }

    d
}

#[no_mangle]
unsafe extern "C" fn stpncpy(
    mut d: *mut c_char,
    mut s: *const c_char,
    mut n: usize,
) -> *mut c_char {
    libc!(libc::stpncpy(d, s, n));

    while n > 0 {
        n -= 1;

        *d = *s;

        if *d == NUL {
            break;
        }

        d = d.add(1);
        s = s.add(1);
    }

    for _ in 0..n {
        *d = 0;
        d = d.add(1);
    }

    d
}

#[no_mangle]
unsafe extern "C" fn strcat(d: *mut c_char, s: *const c_char) -> *mut c_char {
    libc!(libc::strcat(d, s));

    strcpy(strchr(d, 0), s);
    d
}

#[no_mangle]
unsafe extern "C" fn strchr(s: *const c_char, c: c_int) -> *mut c_char {
    libc!(libc::strchr(s, c));

    let mut s = s as *mut c_char;
    while *s != NUL {
        if *s == c as _ {
            return s;
        }
        s = s.add(1);
    }

    ptr::null_mut()
}

#[no_mangle]
unsafe extern "C" fn strcmp(mut s1: *const c_char, mut s2: *const c_char) -> c_int {
    libc!(libc::strcmp(s1, s2));

    while *s1 != NUL && *s2 != NUL {
        if *s1 != *s2 {
            break;
        }

        s1 = s1.add(1);
        s2 = s2.add(1);
    }

    *s1 as c_schar as c_int - *s2 as c_schar as c_int
}

#[no_mangle]
unsafe extern "C" fn strcpy(d: *mut c_char, s: *const c_char) -> *mut c_char {
    libc!(libc::strcpy(d, s));

    stpcpy(d, s);
    d
}

#[no_mangle]
unsafe extern "C" fn strcspn(s: *const c_char, m: *const c_char) -> usize {
    libc!(libc::strspn(s, m));

    let mut w = s;
    while *w != NUL {
        let mut m = m;
        while *m != NUL {
            if *w == *m {
                break;
            }
            m = m.add(1);
        }

        if *m != NUL {
            break;
        }

        w = w.add(1);
    }

    w.offset_from(s) as usize
}

#[no_mangle]
unsafe extern "C" fn strdup(s: *const c_char) -> *mut c_char {
    libc!(libc::strdup(s));

    let len = strlen(s);
    let d = malloc(len + 1);
    if !d.is_null() {
        memcpy(d, s.cast(), len + 1);
    }
    d.cast()
}

#[no_mangle]
unsafe extern "C" fn strlen(s: *const c_char) -> usize {
    libc!(libc::strlen(s));

    let mut w = s;
    while *w != NUL {
        w = w.add(1);
    }

    w.offset_from(s) as usize
}

#[no_mangle]
unsafe extern "C" fn strncat(d: *mut c_char, mut s: *const c_char, mut n: usize) -> *mut c_char {
    libc!(libc::strncat(d, s, n));

    let mut w = strchr(d, 0);

    while n > 0 && *s != NUL {
        n -= 1;

        *w = *s;

        w = w.add(1);
        s = s.add(1);
    }
    *w = 0;

    d
}

#[no_mangle]
unsafe extern "C" fn strncmp(mut s1: *const c_char, mut s2: *const c_char, mut n: usize) -> c_int {
    libc!(libc::strncmp(s1, s2, n));

    while n > 0 && *s1 != NUL && *s2 != NUL {
        n -= 1;

        if *s1 != *s2 {
            break;
        }

        s1 = s1.add(1);
        s2 = s2.add(1);
    }

    *s1 as c_schar as c_int - *s2 as c_schar as c_int
}

#[no_mangle]
unsafe extern "C" fn strndup(s: *const c_char, n: usize) -> *mut c_char {
    libc!(libc::strndup(s, n));

    let len = strnlen(s, n);
    let d = malloc(len + 1);
    if !d.is_null() {
        memcpy(d, s.cast(), len);
    }

    let ret = d.cast::<c_char>();
    *ret.add(len) = 0;
    ret
}

#[no_mangle]
unsafe extern "C" fn strnlen(s: *const c_char, mut n: usize) -> usize {
    libc!(libc::strlen(s));

    let mut w = s;
    while n > 0 && *w != NUL {
        n -= 1;
        w = w.add(1);
    }

    w.offset_from(s) as usize
}

#[no_mangle]
unsafe extern "C" fn strpbrk(s: *const c_char, m: *const c_char) -> *mut c_char {
    libc!(libc::strpbrk(s, m));

    let s = s.add(strcspn(s, m)) as *mut _;

    if *s != NUL {
        return ptr::null_mut();
    }

    s
}

#[no_mangle]
unsafe extern "C" fn strrchr(s: *const c_char, c: c_int) -> *mut c_char {
    libc!(libc::strrchr(s, c));

    let mut s = s as *mut c_char;
    let mut ret = ptr::null_mut::<c_char>();
    loop {
        s = strchr(s, c);
        if s.is_null() {
            break;
        }

        ret = s;
        s = s.add(1);
    }

    ret
}

#[no_mangle]
unsafe extern "C" fn strspn(s: *const c_char, m: *const c_char) -> usize {
    libc!(libc::strspn(s, m));

    let mut w = s;
    while *w != NUL {
        let mut m = m;
        while *m != NUL {
            if *w == *m {
                break;
            }
            m = m.add(1);
        }

        if *m == NUL {
            break;
        }

        w = w.add(1);
    }

    w.offset_from(s) as usize
}

#[no_mangle]
unsafe extern "C" fn strtok(s: *mut c_char, m: *const c_char) -> *mut c_char {
    libc!(libc::strtok(s, m));

    #[repr(transparent)]
    #[derive(Copy, Clone)]
    struct SyncCChar(*mut c_char);
    unsafe impl Sync for SyncCChar {}

    static STORAGE: SyncUnsafeCell<SyncCChar> = SyncUnsafeCell::new(SyncCChar(ptr::null_mut()));

    strtok_r(s, m, SyncUnsafeCell::get(&STORAGE) as *mut *mut c_char)
}

#[no_mangle]
unsafe extern "C" fn strtok_r(
    s: *mut c_char,
    m: *const c_char,
    p: *mut *mut c_char,
) -> *mut c_char {
    libc!(libc::strtok_r(s, m, p));

    let mut s = if s.is_null() { *p } else { s };

    if s.is_null() {
        return ptr::null_mut();
    }

    s = s.add(strspn(s, m));
    if *s == NUL {
        *p = ptr::null_mut();
        return ptr::null_mut();
    }

    let t = s.add(strcspn(s, m));
    if *t != NUL {
        *t = NUL;
        *p = t;
    } else {
        *p = ptr::null_mut();
    }

    s
}
