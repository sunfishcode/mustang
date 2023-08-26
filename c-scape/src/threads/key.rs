use alloc::boxed::Box;
use core::cell::Cell;
use core::ptr::{null, null_mut};
use core::sync::atomic::{AtomicU32, Ordering};
use errno::{set_errno, Errno};
use libc::{c_int, c_void};

use rustix_futex_sync::RwLock;

#[cfg(target_env = "gnu")]
const PTHREAD_KEYS_MAX: u32 = 1024;
const PTHREAD_DESTRUCTOR_ITERATIONS: u8 = 4;

#[derive(Clone, Copy)]
struct KeyData {
    next_key: libc::pthread_key_t,
    destructors: [Option<unsafe extern "C" fn(_: *mut c_void)>; PTHREAD_KEYS_MAX as usize],
}

#[derive(Clone, Copy)]
struct ValueWithEpoch {
    epoch: u32,
    data: *mut c_void,
}

impl ValueWithEpoch {
    const fn new() -> Self {
        ValueWithEpoch {
            epoch: 0,
            data: null_mut(),
        }
    }
}

static KEY_DATA: RwLock<KeyData> = RwLock::new(KeyData {
    next_key: 0,
    destructors: [None; PTHREAD_KEYS_MAX as usize],
});

static EPOCHS: [AtomicU32; PTHREAD_KEYS_MAX as usize] =
    [const { AtomicU32::new(0) }; PTHREAD_KEYS_MAX as usize];

// This uses an epoch-based system for differentiating between
// reused keys corresponding to the same slot.
#[thread_local]
static VALUES: [Cell<ValueWithEpoch>; PTHREAD_KEYS_MAX as usize] =
    [const { Cell::new(ValueWithEpoch::new()) }; PTHREAD_KEYS_MAX as usize];

#[thread_local]
static HAS_REGISTERED_CLEANUP: Cell<bool> = Cell::new(false);

#[no_mangle]
unsafe extern "C" fn pthread_getspecific(key: libc::pthread_key_t) -> *mut c_void {
    libc!(libc::pthread_getspecific(key));

    let latest_epoch = EPOCHS[key as usize].load(Ordering::SeqCst);
    let ValueWithEpoch { epoch, data } = VALUES[key as usize].get();

    // If the latest epoch is newer, then that means this slot got reallocated.
    // Either we are reusing a deleted key, which is UB, or we are reusing
    // the new key, which must return null initially.
    if epoch < latest_epoch {
        null_mut()
    } else {
        data
    }
}

#[no_mangle]
unsafe extern "C" fn pthread_setspecific(key: libc::pthread_key_t, value: *const c_void) -> c_int {
    libc!(libc::pthread_setspecific(key, value));

    // If this is the first-time we have gotten here,
    // we need to actually register the dtors for cleanup.
    if !HAS_REGISTERED_CLEANUP.get() {
        origin::at_thread_exit(Box::new(move || {
            for _ in 0..PTHREAD_DESTRUCTOR_ITERATIONS {
                let mut ran_dtor = false;

                for i in 0..PTHREAD_KEYS_MAX {
                    let data = pthread_getspecific(i as libc::pthread_key_t);

                    if data.is_null() {
                        continue;
                    }

                    ran_dtor = true;

                    let dtor = {
                        // POSIX says that `pthread_key_delete` can
                        // be called within a destructor function...
                        // We have to take each dtor one at a time just
                        // in case someone did delete it;
                        let key_data = KEY_DATA.read();
                        key_data.destructors[i as usize]
                    };

                    if let Some(dtor) = dtor {
                        // Null out the data as required
                        // by POSIX semantics. This may bump
                        // the local epoch, but that doesn't matter.
                        pthread_setspecific(i as libc::pthread_key_t, null());

                        // Call the destructor with the old
                        // data, before we set it to null.
                        dtor(value as *mut _);
                    }
                }

                if !ran_dtor {
                    break;
                }
            }
        }));

        HAS_REGISTERED_CLEANUP.set(true);
    }

    VALUES[key as usize].set(ValueWithEpoch {
        epoch: EPOCHS[key as usize].load(Ordering::SeqCst),
        data: value as *mut _,
    });
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_key_create(
    key: *mut libc::pthread_key_t,
    dtor: Option<unsafe extern "C" fn(_: *mut c_void)>,
) -> c_int {
    libc!(libc::pthread_key_create(key, dtor));

    extern "C" fn empty_dtor(_: *mut c_void) {}

    let mut key_data = KEY_DATA.write();

    let mut next_key = key_data.next_key;
    if next_key < PTHREAD_KEYS_MAX {
        // Fast-path, less than `PTHREAD_KEYS_MAX` slots
        // have been allocated, we just use this as a bump
        // allocator basically.
        key_data.next_key = next_key + 1;
    } else {
        // Slow-path, linearly scan through the table to try
        // and find an empty slot.
        for (index, dtor) in key_data.destructors.iter().enumerate() {
            if dtor.is_none() {
                // We have to bump the epoch now that we are
                // reusing slots.
                if EPOCHS[index].fetch_add(1, Ordering::SeqCst) == 0 {
                    panic!("detected epoch counter overflow");
                }
                next_key = index as libc::pthread_key_t;
            }
        }

        // If the loop still did not find a valid key
        if next_key >= PTHREAD_KEYS_MAX {
            set_errno(Errno(libc::EAGAIN));
            return -1;
        }
    }

    // We have to `unwrap_or` the dtor because `None` is reserved for signifying
    // that the key is not allocated.
    key_data.destructors[next_key as usize] = Some(dtor.unwrap_or(empty_dtor));

    0
}

#[no_mangle]
unsafe extern "C" fn pthread_key_delete(key: libc::pthread_key_t) -> c_int {
    libc!(libc::pthread_key_delete(key));

    let mut key_data = KEY_DATA.write();
    key_data.destructors[key as usize] = None;

    0
}
