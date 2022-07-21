use alloc::boxed::Box;
use core::cell::Cell;
use core::ptr::null_mut;
use errno::{set_errno, Errno};
use libc::{c_int, c_void};

use origin::sync::Mutex;

const PTHREAD_KEYS_MAX: u32 = 128;
const PTHREAD_DESTRUCTOR_ITERATIONS: u8 = 4;

#[derive(Clone, Copy)]
struct KeyData {
    current_key: libc::pthread_key_t,
    destructors: [Option<unsafe extern "C" fn(_: *mut c_void)>; PTHREAD_KEYS_MAX as usize],
}

static KEY_DATA: Mutex<KeyData> = Mutex::new(KeyData {
    current_key: 0,
    destructors: [None; PTHREAD_KEYS_MAX as usize],
});

const VALUES_INIT: Cell<*mut c_void> = Cell::new(null_mut());

#[thread_local]
static VALUES: [Cell<*mut c_void>; PTHREAD_KEYS_MAX as usize] =
    [VALUES_INIT; PTHREAD_KEYS_MAX as usize];

#[thread_local]
static HAS_REGISTERED_CLEANUP: Cell<bool> = Cell::new(false);

#[no_mangle]
unsafe extern "C" fn pthread_getspecific(key: libc::pthread_key_t) -> *mut c_void {
    libc!(libc::pthread_getspecific(key));

    VALUES[key as usize].get()
}

#[no_mangle]
unsafe extern "C" fn pthread_setspecific(key: libc::pthread_key_t, value: *const c_void) -> c_int {
    libc!(libc::pthread_setspecific(key, value));

    if !HAS_REGISTERED_CLEANUP.get() {
        // TODO: The ordering that we need here is
        // all the pthread_cleanup functions,
        // then all thread-local dtors,
        // and then finally all static
        // (`__cxa_thread_atexit_impl`) dtors.
        //
        // We don't have the mechanism for that yet though.
        origin::at_thread_exit(Box::new(move || {
            for _ in 0..PTHREAD_DESTRUCTOR_ITERATIONS {
                let mut ran_dtor = false;

                for i in 0..PTHREAD_KEYS_MAX {
                    let value_ptr = VALUES[i as usize].get();

                    if value_ptr.is_null() {
                        continue;
                    }

                    ran_dtor = true;

                    let dtor = {
                        // POSIX says that `pthread_key_delete` can
                        // be called within a destructor function...
                        // We have to take each dtor one at a time just
                        // in case someone did delete it;
                        let key_data = KEY_DATA.lock();
                        key_data.destructors[i as usize]
                    };

                    if let Some(dtor) = dtor {
                        VALUES[i as usize].set(null_mut());
                        dtor(value_ptr);
                    }
                }

                if !ran_dtor {
                    break;
                }
            }
        }));

        HAS_REGISTERED_CLEANUP.set(true);
    }

    VALUES[key as usize].set(value as *mut _);
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_key_create(
    key: *mut libc::pthread_key_t,
    dtor: Option<unsafe extern "C" fn(_: *mut c_void)>,
) -> c_int {
    libc!(libc::pthread_key_create(key, dtor));

    let mut key_data = KEY_DATA.lock();

    let next_key = key_data.current_key;
    if next_key >= PTHREAD_KEYS_MAX {
        set_errno(Errno(libc::EAGAIN));
        return -1;
    }

    key_data.current_key = next_key + 1;
    key_data.destructors[next_key as usize] = dtor;

    0
}

#[no_mangle]
unsafe extern "C" fn pthread_key_delete(key: libc::pthread_key_t) -> c_int {
    libc!(libc::pthread_key_delete(key));

    let mut key_data = KEY_DATA.lock();
    key_data.destructors[key as usize] = None;

    0
}
