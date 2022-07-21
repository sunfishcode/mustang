use core::cell::Cell;
use core::ptr::{null, null_mut};
use errno::{set_errno, Errno};
use libc::{c_int, c_void};

const PTHREAD_KEYS_MAX: u32 = 128;

#[thread_local]
static CURRENT_KEY: Cell<libc::pthread_key_t> = Cell::new(0);

#[derive(Clone, Copy)]
struct KeyData {
    data: *mut c_void,
    dtor: Option<unsafe extern "C" fn(_: *mut c_void)>,
}

impl KeyData {
    const fn new() -> KeyData {
        KeyData {
            data: null_mut(),
            dtor: None,
        }
    }
}

#[thread_local]
static VALUES: [Cell<KeyData>; PTHREAD_KEYS_MAX as usize] = [Cell::new(KeyData::new()); PTHREAD_KEYS_MAX as usize];

#[no_mangle]
unsafe extern "C" fn pthread_getspecific(
    key: libc::pthread_key_t
) -> *mut c_void {
    libc!(libc::pthread_getspecific(key));

    let current_cell = &VALUES.as_array_of_cells()[key as usize];

    let temp = current_cell.get();
    temp.data
}

#[no_mangle]
unsafe extern "C" fn pthread_setspecific(
    key: libc::pthread_key_t, 
    value: *const c_void
) -> c_int {
    libc!(libc::pthread_setspecific(key, value));

    let current_cell = &VALUES.as_array_of_cells()[key as usize];

    let mut temp = current_cell.get();
    temp.data = value as *mut _;
    current_cell.set(temp);

    0
}

#[no_mangle]
unsafe extern "C" fn pthread_key_create(
    key: *mut libc::pthread_key_t, 
    dtor: Option<unsafe extern "C" fn(_: *mut c_void)>
) -> c_int {
    libc!(libc::pthread_key_create(key, dtor));

    let next_key = CURRENT_KEY.get();
    if next_key >= PTHREAD_KEYS_MAX {
        set_errno(Errno(libc::EAGAIN));
        return -1;
    }
    CURRENT_KEY.set(next_key + 1);
    
    
    let current_cell = &VALUES.as_array_of_cells()[next_key as usize];

    let mut temp = current_cell.get();
    temp.dtor = dtor;
    current_cell.set(temp);
    
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_key_delete(
    key: libc::pthread_key_t
) -> c_int {
    libc!(libc::pthread_key_delete(key));

    pthread_setspecific(key, null())
}
