mustang::can_run_this!();

use std::ffi::{CStr, OsStr, OsString};
use std::os::raw::{c_char, c_int};
use std::os::unix::ffi::OsStrExt;
use std::slice;
use std::sync::atomic::{AtomicBool, Ordering};

static EARLY_CTOR_INITIALIZED: AtomicBool = AtomicBool::new(false);
static CTOR_INITIALIZED: AtomicBool = AtomicBool::new(false);
static MANUAL_CTOR_INITIALIZED: AtomicBool = AtomicBool::new(false);
static DTOR_PERFORMED: AtomicBool = AtomicBool::new(false);

#[ctor::ctor]
fn ctor() {
    assert!(EARLY_CTOR_INITIALIZED.load(Ordering::Relaxed));
    CTOR_INITIALIZED.store(true, Ordering::Relaxed);
}

fn main() {
    assert!(EARLY_CTOR_INITIALIZED.load(Ordering::Relaxed));
    assert!(CTOR_INITIALIZED.load(Ordering::Relaxed));
    assert!(MANUAL_CTOR_INITIALIZED.load(Ordering::Relaxed));
}

#[ctor::dtor]
fn dtor() {
    DTOR_PERFORMED.store(true, Ordering::Relaxed);
}

#[link_section = ".init_array"]
#[used]
static INIT_ARRAY: unsafe extern "C" fn(c_int, *mut *mut c_char, *mut *mut c_char) = {
    unsafe extern "C" fn function(argc: c_int, argv: *mut *mut c_char, envp: *mut *mut c_char) {
        assert_eq!(argc as usize, std::env::args_os().len());

        assert_eq!(
            slice::from_raw_parts(argv, argc as usize)
                .iter()
                .map(|arg| OsStr::from_bytes(CStr::from_ptr(*arg).to_bytes()).to_owned())
                .collect::<Vec<OsString>>(),
            std::env::args_os().collect::<Vec<OsString>>()
        );
        assert_eq!(*argv.add(argc as usize), core::ptr::null_mut());

        assert_ne!(envp, core::ptr::null_mut());

        let mut ptr = envp;
        let mut num_env = 0;
        loop {
            let env = *ptr;
            if env.is_null() {
                break;
            }

            let bytes = CStr::from_ptr(env).to_bytes();
            let mut parts = bytes.splitn(2, |byte| *byte == b'=');
            let key = parts.next().unwrap();
            let value = parts.next().unwrap();
            assert_eq!(
                std::env::var_os(OsStr::from_bytes(key)).expect("missing environment variable"),
                OsStr::from_bytes(value)
            );

            num_env += 1;
            ptr = ptr.add(1);
        }
        assert_eq!(num_env, std::env::vars_os().count());

        MANUAL_CTOR_INITIALIZED.store(true, Ordering::Relaxed);
    }
    function
};

#[link_section = ".init_array.00000"]
#[used]
static EARLY_INIT_ARRAY: unsafe extern "C" fn(c_int, *mut *mut c_char, *mut *mut c_char) = {
    unsafe extern "C" fn function(_argc: c_int, _argv: *mut *mut c_char, _envp: *mut *mut c_char) {
        EARLY_CTOR_INITIALIZED.store(true, Ordering::Relaxed);
        assert!(!CTOR_INITIALIZED.load(Ordering::Relaxed));
        assert!(!MANUAL_CTOR_INITIALIZED.load(Ordering::Relaxed));
    }
    function
};

#[link_section = ".fini_array.00000"]
#[used]
static LATE_FINI_ARRAY: unsafe extern "C" fn(c_int, *mut *mut c_char, *mut *mut c_char) = {
    unsafe extern "C" fn function(_argc: c_int, _argv: *mut *mut c_char, _envp: *mut *mut c_char) {
        assert!(DTOR_PERFORMED.load(Ordering::Relaxed));
    }
    function
};
