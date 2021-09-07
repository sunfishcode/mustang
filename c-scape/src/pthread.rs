//! Pthread functionality is stubbed out. This is sort of just enough for
//! programs that don't create new threads.

#![allow(non_camel_case_types)]

use std::os::raw::{c_int, c_void};
use std::ptr::null;

pub type pthread_t = usize;

#[no_mangle]
pub unsafe extern "C" fn pthread_self() -> pthread_t {
    1
}

#[no_mangle]
pub unsafe extern "C" fn pthread_getattr_np(_thread: pthread_t, _attr: *mut c_void) -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_attr_init(_attr: *mut c_void) -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_attr_destroy(_attr: *mut c_void) -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_attr_getstack(
    _attr: *const c_void,
    stackaddr: *mut *mut c_void,
    stacksize: *mut usize,
) -> c_int {
    *stackaddr = rsix::process::page_size() as _;
    *stacksize = rsix::process::page_size();
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_getspecific() -> *const c_void {
    null()
}

#[no_mangle]
pub unsafe extern "C" fn pthread_key_create() -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_key_delete() -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_mutexattr_destroy() -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_mutexattr_init() -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_mutexattr_settype() -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_destroy() -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_init() -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_lock() -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_trylock() -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_unlock() -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_rdlock() -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_unlock() -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_setspecific() -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_attr_getguardsize() -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_attr_setguardsize() -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_create() -> c_int {
    unimplemented!("pthread_create")
}

#[no_mangle]
pub unsafe extern "C" fn pthread_detach() -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_join() -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_sigmask() -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_attr_setstacksize() -> c_int {
    0
}
