use core::convert::TryInto;
use core::ffi::c_void;
use core::ptr::{self, null, null_mut};
use core::sync::atomic::Ordering::SeqCst;
use core::sync::atomic::{AtomicBool, AtomicU32};
use parking_lot::lock_api::{RawMutex as _, RawRwLock};
use parking_lot::RawMutex;

use libc::c_int;

struct GetThreadId;

unsafe impl parking_lot::lock_api::GetThreadId for GetThreadId {
    const INIT: Self = Self;

    fn nonzero_thread_id(&self) -> core::num::NonZeroUsize {
        origin::current_thread_id()
            .as_raw_nonzero()
            .try_into()
            .unwrap()
    }
}

// In Linux, `pthread_t` is usually `unsigned long`, but we make it a pointer
// type so that it preserves provenance.
#[allow(non_camel_case_types)]
type PthreadT = *mut c_void;
libc_type!(PthreadT, pthread_t);

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Clone)]
struct PthreadAttrT {
    stack_addr: *mut c_void,
    stack_size: usize,
    guard_size: usize,
    pad0: usize,
    pad1: usize,
    pad2: usize,
    pad3: usize,
    #[cfg(target_arch = "x86")]
    pad4: usize,
    #[cfg(target_arch = "x86")]
    pad5: usize,
}
libc_type!(PthreadAttrT, pthread_attr_t);

impl Default for PthreadAttrT {
    fn default() -> Self {
        Self {
            stack_addr: null_mut(),
            stack_size: origin::default_stack_size(),
            guard_size: origin::default_guard_size(),
            pad0: 0,
            pad1: 0,
            pad2: 0,
            pad3: 0,
            #[cfg(target_arch = "x86")]
            pad4: 0,
            #[cfg(target_arch = "x86")]
            pad5: 0,
        }
    }
}

#[allow(non_camel_case_types)]
#[repr(C)]
union pthread_mutex_u {
    // Use a custom `RawMutex` for non-reentrant pthread mutex for now, because
    // we use `dlmalloc` or `wee_alloc` for global allocations, which use
    // mutexes, and `parking_lot`'s `RawMutex` performs global allocations.
    normal: RawMutex,
    reentrant: parking_lot::lock_api::RawReentrantMutex<parking_lot::RawMutex, GetThreadId>,
}

#[allow(non_camel_case_types)]
#[repr(C)]
struct PthreadMutexT {
    kind: AtomicU32,
    u: pthread_mutex_u,
    pad0: usize,
    #[cfg(target_arch = "x86")]
    pad1: usize,
}
libc_type!(PthreadMutexT, pthread_mutex_t);

#[allow(non_camel_case_types)]
#[repr(C)]
struct PthreadRwlockT {
    lock: parking_lot::RawRwLock,
    exclusive: AtomicBool,
    pad0: usize,
    pad1: usize,
    pad2: usize,
    pad3: usize,
    pad4: usize,
    #[cfg(target_arch = "x86")]
    pad5: usize,
}
libc_type!(PthreadRwlockT, pthread_rwlock_t);

#[allow(non_camel_case_types)]
#[repr(C)]
struct PthreadMutexattrT {
    kind: AtomicU32,
}
libc_type!(PthreadMutexattrT, pthread_mutexattr_t);

#[no_mangle]
unsafe extern "C" fn pthread_self() -> PthreadT {
    libc!(ptr::from_exposed_addr_mut(libc::pthread_self() as _));
    origin::current_thread().cast()
}

#[no_mangle]
unsafe extern "C" fn pthread_getattr_np(thread: PthreadT, attr: *mut PthreadAttrT) -> c_int {
    // FIXME(#95) layout of attr doesn't match signature on aarch64
    // uncomment once it does:
    // libc!(libc::pthread_getattr_np(thread, checked_cast!(attr)));
    let (stack_addr, stack_size, guard_size) = origin::thread_stack(thread.cast());
    ptr::write(
        attr,
        PthreadAttrT {
            stack_addr,
            stack_size,
            guard_size,
            pad0: 0,
            pad1: 0,
            pad2: 0,
            pad3: 0,
            #[cfg(target_arch = "x86")]
            pad4: 0,
            #[cfg(target_arch = "x86")]
            pad5: 0,
        },
    );
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_attr_init(attr: *mut PthreadAttrT) -> c_int {
    // FIXME(#95) layout of attr doesn't match signature on aarch64
    // uncomment once it does:
    // libc!(libc::pthread_attr_init(checked_cast!(attr)));
    ptr::write(attr, PthreadAttrT::default());
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_attr_destroy(attr: *mut PthreadAttrT) -> c_int {
    // FIXME(#95) layout of attr doesn't match signature on aarch64
    // uncomment once it does:
    // libc!(libc::pthread_attr_destroy(checked_cast!(attr)));
    ptr::drop_in_place(attr);
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_attr_getstack(
    attr: *const PthreadAttrT,
    stackaddr: *mut *mut c_void,
    stacksize: *mut usize,
) -> c_int {
    // FIXME(#95) layout of attr doesn't match signature on aarch64
    // uncomment once it does:
    // libc!(libc::pthread_attr_getstack(checked_cast!(attr), stackaddr, stacksize));
    *stackaddr = (*attr).stack_addr;
    *stacksize = (*attr).stack_size;
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_getspecific() -> *const c_void {
    //libc!(libc::pthread_getspecific());
    rustix::io::write(
        &rustix::io::stderr(),
        b"unimplemented: pthread_getspecific\n",
    )
    .ok();
    null()
}

#[no_mangle]
unsafe extern "C" fn pthread_key_create() -> c_int {
    //libc!(libc::pthread_key_create());
    rustix::io::write(
        &rustix::io::stderr(),
        b"unimplemented: pthread_key_create\n",
    )
    .ok();
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_key_delete() -> c_int {
    //libc!(libc::pthread_key_delete());
    rustix::io::write(
        &rustix::io::stderr(),
        b"unimplemented: pthread_key_delete\n",
    )
    .ok();
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_mutexattr_destroy(attr: *mut PthreadMutexattrT) -> c_int {
    // FIXME(#95) layout of attr doesn't match signature on aarch64
    // uncomment once it does:
    // libc!(libc::pthread_mutexattr_destroy(checked_cast!(attr)));
    ptr::drop_in_place(attr);
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_mutexattr_init(attr: *mut PthreadMutexattrT) -> c_int {
    // FIXME(#95) layout of attr doesn't match signature on aarch64
    // uncomment once it does:
    // libc!(libc::pthread_mutexattr_init(checked_cast!(attr)));
    ptr::write(
        attr,
        PthreadMutexattrT {
            kind: AtomicU32::new(libc::PTHREAD_MUTEX_DEFAULT as u32),
        },
    );
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_mutexattr_settype(attr: *mut PthreadMutexattrT, kind: c_int) -> c_int {
    // FIXME(#95) layout of attr doesn't match signature on aarch64
    // uncomment once it does:
    // libc!(libc::pthread_mutexattr_settype(checked_cast!(attr), kind));
    (*attr).kind = AtomicU32::new(kind as u32);
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_mutex_destroy(mutex: *mut PthreadMutexT) -> c_int {
    // FIXME(#95) layout of mutex doesn't match signature on aarch64
    // uncomment once it does:
    // libc!(libc::pthread_mutex_destroy(checked_cast!(mutex)));
    match (*mutex).kind.load(SeqCst) as i32 {
        libc::PTHREAD_MUTEX_NORMAL => ptr::drop_in_place(&mut (*mutex).u.normal),
        libc::PTHREAD_MUTEX_RECURSIVE => ptr::drop_in_place(&mut (*mutex).u.reentrant),
        libc::PTHREAD_MUTEX_ERRORCHECK => unimplemented!("PTHREAD_MUTEX_ERRORCHECK"),
        other => unimplemented!("unsupported pthread mutex kind {}", other),
    }
    (*mutex).kind.store(!0, SeqCst);
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_mutex_init(
    mutex: *mut PthreadMutexT,
    mutexattr: *const PthreadMutexattrT,
) -> c_int {
    // FIXME(#95) layout of mutex and attr doesn't match signature on aarch64
    // uncomment once it does:
    // libc!(libc::pthread_mutex_init(checked_cast!(mutex), checked_cast!(mutexattr)));
    let kind = (*mutexattr).kind.load(SeqCst);
    match kind as i32 {
        libc::PTHREAD_MUTEX_NORMAL => ptr::write(&mut (*mutex).u.normal, RawMutex::INIT),
        libc::PTHREAD_MUTEX_RECURSIVE => ptr::write(
            &mut (*mutex).u.reentrant,
            parking_lot::lock_api::RawReentrantMutex::<parking_lot::RawMutex, GetThreadId>::INIT,
        ),
        libc::PTHREAD_MUTEX_ERRORCHECK => unimplemented!("PTHREAD_MUTEX_ERRORCHECK"),
        other => unimplemented!("unsupported pthread mutex kind {}", other),
    }
    (*mutex).kind.store(kind, SeqCst);
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_mutex_lock(mutex: *mut PthreadMutexT) -> c_int {
    // FIXME(#95) layout of mutex doesn't match signature on aarch64
    // uncomment once it does:
    // libc!(libc::pthread_mutex_lock(checked_cast!(mutex)));
    match (*mutex).kind.load(SeqCst) as i32 {
        libc::PTHREAD_MUTEX_NORMAL => (*mutex).u.normal.lock(),
        libc::PTHREAD_MUTEX_RECURSIVE => (*mutex).u.reentrant.lock(),
        libc::PTHREAD_MUTEX_ERRORCHECK => unimplemented!("PTHREAD_MUTEX_ERRORCHECK"),
        other => unimplemented!("unsupported pthread mutex kind {}", other),
    }
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_mutex_trylock(mutex: *mut PthreadMutexT) -> c_int {
    // FIXME(#95) layout of mutex doesn't match signature on aarch64
    // uncomment once it does:
    // libc!(libc::pthread_mutex_trylock(checked_cast!(mutex)));
    if match (*mutex).kind.load(SeqCst) as i32 {
        libc::PTHREAD_MUTEX_NORMAL => (*mutex).u.normal.try_lock(),
        libc::PTHREAD_MUTEX_RECURSIVE => (*mutex).u.reentrant.try_lock(),
        libc::PTHREAD_MUTEX_ERRORCHECK => unimplemented!("PTHREAD_MUTEX_ERRORCHECK"),
        other => unimplemented!("unsupported pthread mutex kind {}", other),
    } {
        0
    } else {
        rustix::io::Error::BUSY.raw_os_error()
    }
}

#[no_mangle]
unsafe extern "C" fn pthread_mutex_unlock(mutex: *mut PthreadMutexT) -> c_int {
    // FIXME(#95) layout of mutex doesn't match signature on aarch64
    // uncomment once it does:
    // libc!(libc::pthread_mutex_unlock(checked_cast!(mutex)));
    match (*mutex).kind.load(SeqCst) as i32 {
        libc::PTHREAD_MUTEX_NORMAL => (*mutex).u.normal.unlock(),
        libc::PTHREAD_MUTEX_RECURSIVE => (*mutex).u.reentrant.unlock(),
        libc::PTHREAD_MUTEX_ERRORCHECK => unimplemented!("PTHREAD_MUTEX_ERRORCHECK"),
        other => unimplemented!("unsupported pthread mutex kind {}", other),
    }
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_rwlock_tryrdlock(rwlock: *mut PthreadRwlockT) -> c_int {
    libc!(libc::pthread_rwlock_tryrdlock(checked_cast!(rwlock)));
    let result = (*rwlock).lock.try_lock_shared();
    if result {
        (*rwlock).exclusive.store(false, SeqCst);
        0
    } else {
        rustix::io::Error::BUSY.raw_os_error()
    }
}

#[no_mangle]
unsafe extern "C" fn pthread_rwlock_trywrlock(rwlock: *mut PthreadRwlockT) -> c_int {
    libc!(libc::pthread_rwlock_trywrlock(checked_cast!(rwlock)));
    let result = (*rwlock).lock.try_lock_exclusive();
    if result {
        (*rwlock).exclusive.store(true, SeqCst);
        0
    } else {
        rustix::io::Error::BUSY.raw_os_error()
    }
}

#[no_mangle]
unsafe extern "C" fn pthread_rwlock_rdlock(rwlock: *mut PthreadRwlockT) -> c_int {
    libc!(libc::pthread_rwlock_rdlock(checked_cast!(rwlock)));
    (*rwlock).lock.lock_shared();
    (*rwlock).exclusive.store(false, SeqCst);
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_rwlock_unlock(rwlock: *mut PthreadRwlockT) -> c_int {
    libc!(libc::pthread_rwlock_unlock(checked_cast!(rwlock)));
    if (*rwlock).exclusive.load(SeqCst) {
        (*rwlock).lock.unlock_exclusive();
    } else {
        (*rwlock).lock.unlock_shared();
    }
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_setspecific() -> c_int {
    //libc!(libc::pthread_setspecific());
    rustix::io::write(
        &rustix::io::stderr(),
        b"unimplemented: pthread_getspecific\n",
    )
    .ok();
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_attr_getguardsize(
    attr: *const PthreadAttrT,
    guardsize: *mut usize,
) -> c_int {
    // FIXME(#95) layout of attr doesn't match signature on aarch64
    // uncomment once it does:
    // libc!(libc::pthread_attr_getguardsize(checked_cast!(attr), guardsize));
    *guardsize = (*attr).guard_size;
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_attr_setguardsize(attr: *mut PthreadAttrT, guardsize: usize) -> c_int {
    // TODO: libc!(libc::pthread_attr_setguardsize(attr, guardsize));
    (*attr).guard_size = guardsize;
    0
}

const SIZEOF_PTHREAD_COND_T: usize = 48;

#[cfg_attr(target_arch = "x86", repr(C, align(4)))]
#[cfg_attr(not(target_arch = "x86"), repr(C, align(8)))]
struct PthreadCondT {
    inner: parking_lot::Condvar,
    pad: [u8; SIZEOF_PTHREAD_COND_T - core::mem::size_of::<parking_lot::Condvar>()],
}

libc_type!(PthreadCondT, pthread_cond_t);

#[cfg(any(
    target_arch = "x86",
    target_arch = "x86_64",
    target_arch = "arm",
    all(target_arch = "aarch64", target_pointer_width = "32"),
    target_arch = "riscv64",
))]
const SIZEOF_PTHREAD_CONDATTR_T: usize = 4;

#[cfg(all(target_arch = "aarch64", target_pointer_width = "64"))]
const SIZEOF_PTHREAD_CONDATTR_T: usize = 8;

#[repr(C, align(4))]
struct PthreadCondattrT {
    pad: [u8; SIZEOF_PTHREAD_CONDATTR_T],
}

libc_type!(PthreadCondattrT, pthread_condattr_t);

#[no_mangle]
unsafe extern "C" fn pthread_condattr_destroy(attr: *mut PthreadCondattrT) -> c_int {
    libc!(libc::pthread_condattr_destroy(checked_cast!(attr)));
    let _ = attr;
    rustix::io::write(
        &rustix::io::stderr(),
        b"unimplemented: pthread_condattr_destroy\n",
    )
    .ok();
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_condattr_init(attr: *mut PthreadCondattrT) -> c_int {
    libc!(libc::pthread_condattr_init(checked_cast!(attr)));
    let _ = attr;
    rustix::io::write(
        &rustix::io::stderr(),
        b"unimplemented: pthread_condattr_init\n",
    )
    .ok();
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_condattr_setclock(
    attr: *mut PthreadCondattrT,
    clock_id: c_int,
) -> c_int {
    libc!(libc::pthread_condattr_setclock(
        checked_cast!(attr),
        clock_id
    ));
    let _ = (attr, clock_id);
    rustix::io::write(
        &rustix::io::stderr(),
        b"unimplemented: pthread_condattr_setclock\n",
    )
    .ok();
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_cond_broadcast(cond: *mut PthreadCondT) -> c_int {
    libc!(libc::pthread_cond_broadcast(checked_cast!(cond)));
    let _ = cond;
    rustix::io::write(
        &rustix::io::stderr(),
        b"unimplemented: pthread_cond_broadcast\n",
    )
    .ok();
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_cond_destroy(cond: *mut PthreadCondT) -> c_int {
    libc!(libc::pthread_cond_destroy(checked_cast!(cond)));
    let _ = cond;
    rustix::io::write(
        &rustix::io::stderr(),
        b"unimplemented: pthread_cond_destroy\n",
    )
    .ok();
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_cond_init(
    cond: *mut PthreadCondT,
    attr: *const PthreadCondattrT,
) -> c_int {
    libc!(libc::pthread_cond_init(
        checked_cast!(cond),
        checked_cast!(attr)
    ));
    let _ = (cond, attr);
    rustix::io::write(&rustix::io::stderr(), b"unimplemented: pthread_cond_init\n").ok();
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_cond_signal(cond: *mut PthreadCondT) -> c_int {
    libc!(libc::pthread_cond_signal(checked_cast!(cond)));
    let _ = cond;
    rustix::io::write(
        &rustix::io::stderr(),
        b"unimplemented: pthread_cond_signal\n",
    )
    .ok();
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_cond_wait(cond: *mut PthreadCondT, lock: *mut PthreadMutexT) -> c_int {
    // FIXME(#95) layout of lock doesn't match signature on aarch64
    // uncomment once it does:
    // libc!(libc::pthread_cond_wait(checked_cast!(cond), checked_cast!(lock));
    let _ = (cond, lock);
    rustix::io::write(&rustix::io::stderr(), b"unimplemented: pthread_cond_wait\n").ok();
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_cond_timedwait(
    cond: *mut PthreadCondT,
    lock: *mut PthreadMutexT,
    abstime: *const libc::timespec,
) -> c_int {
    // FIXME(#95) layout of mutex doesn't match signature on aarch64
    // uncomment once it does:
    // libc!(libc::pthread_cond_timedwait(checked_cast!(cond), checked_cast!(lock), abstime));
    let _ = (cond, lock, abstime);
    rustix::io::write(
        &rustix::io::stderr(),
        b"unimplemented: pthread_cond_timedwait\n",
    )
    .ok();
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_rwlock_destroy(rwlock: *mut PthreadRwlockT) -> c_int {
    libc!(libc::pthread_rwlock_destroy(checked_cast!(rwlock)));
    ptr::drop_in_place(rwlock);
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_rwlock_wrlock(rwlock: *mut PthreadRwlockT) -> c_int {
    libc!(libc::pthread_rwlock_wrlock(checked_cast!(rwlock)));
    (*rwlock).lock.lock_exclusive();
    (*rwlock).exclusive.store(true, SeqCst);
    0
}

// TODO: signals, create thread-local storage, arrange for thread-local storage
// destructors to run, free the `mmap`.
#[no_mangle]
unsafe extern "C" fn pthread_create(
    pthread: *mut PthreadT,
    attr: *const PthreadAttrT,
    fn_: extern "C" fn(*mut c_void) -> *mut c_void,
    arg: *mut c_void,
) -> c_int {
    // FIXME(#95) layout of attr doesn't match signature on aarch64
    // uncomment once it does:
    // libc!(libc::pthread_create(pthread, checked_cast!(attr), fn_, arg));
    let PthreadAttrT {
        stack_addr,
        stack_size,
        guard_size,
        pad0: _,
        pad1: _,
        pad2: _,
        pad3: _,
        #[cfg(target_arch = "x86")]
            pad4: _,
        #[cfg(target_arch = "x86")]
            pad5: _,
    } = if attr.is_null() {
        PthreadAttrT::default()
    } else {
        (*attr).clone()
    };
    assert!(stack_addr.is_null());

    let thread = match origin::create_thread(
        Box::new(move || {
            fn_(arg);
            Some(Box::new(arg))
        }),
        stack_size,
        guard_size,
    ) {
        Ok(thread) => thread,
        Err(e) => return e.raw_os_error(),
    };

    pthread.write(thread.cast());
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_detach(pthread: PthreadT) -> c_int {
    libc!(libc::pthread_detach(pthread.expose_addr() as _));
    origin::detach_thread(pthread.cast());
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_join(pthread: PthreadT, retval: *mut *mut c_void) -> c_int {
    libc!(libc::pthread_join(pthread.expose_addr() as _, retval));
    assert!(
        retval.is_null(),
        "pthread_join return values not supported yet"
    );

    origin::join_thread(pthread.cast());
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_sigmask() -> c_int {
    //libc!(libc::pthread_sigmask());
    // not yet implemented, what is needed fo `std::Command` to work.
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_attr_setstacksize(attr: *mut PthreadAttrT, stacksize: usize) -> c_int {
    // FIXME(#95) layout of attr doesn't match signature on aarch64
    // uncomment once it does:
    // libc!(libc::pthread_attr_setstacksize(checked_cast!(attr), stacksize));
    (*attr).stack_size = stacksize;
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_cancel() -> c_int {
    //libc!(libc::pthread_cancel());
    unimplemented!("pthread_cancel")
}

#[no_mangle]
unsafe extern "C" fn pthread_exit() -> c_int {
    //libc!(libc::pthread_exit());
    // As with `pthread_cancel`, `pthread_exit` may be tricky to implement.
    unimplemented!("pthread_exit")
}

#[no_mangle]
unsafe extern "C" fn pthread_cleanup_push() -> c_int {
    //libc!(libc::pthread_cleanup_push());
    unimplemented!("pthread_cleanup_push")
}

#[no_mangle]
unsafe extern "C" fn pthread_cleanup_pop() -> c_int {
    //libc!(libc::pthread_cleanup_pop());
    unimplemented!("pthread_cleanup_pop")
}

#[no_mangle]
unsafe extern "C" fn pthread_setcancelstate() -> c_int {
    //libc!(libc::pthread_setcancelstate());
    unimplemented!("pthread_setcancelstate")
}

#[no_mangle]
unsafe extern "C" fn pthread_setcanceltype() -> c_int {
    //libc!(libc::pthread_setcanceltype());
    unimplemented!("pthread_setcanceltype")
}

#[no_mangle]
unsafe extern "C" fn pthread_testcancel() -> c_int {
    //libc!(libc::pthread_testcancel());
    unimplemented!("pthread_testcancel")
}

#[no_mangle]
unsafe extern "C" fn pthread_atfork(
    prepare: Option<unsafe extern "C" fn()>,
    parent: Option<unsafe extern "C" fn()>,
    child: Option<unsafe extern "C" fn()>,
) -> c_int {
    libc!(libc::pthread_atfork(prepare, parent, child));
    crate::at_fork::at_fork(prepare, parent, child);
    0
}

#[no_mangle]
unsafe extern "C" fn __cxa_thread_atexit_impl(
    func: unsafe extern "C" fn(*mut c_void),
    obj: *mut c_void,
    _dso_symbol: *mut c_void,
) -> c_int {
    // TODO: libc!(libc::__cxa_thread_atexit_impl(func, obj, _dso_symbol));
    origin::at_thread_exit_raw(func, obj);
    0
}

#[cfg(feature = "threads")]
#[no_mangle]
unsafe extern "C" fn __tls_get_addr(p: &[usize; 2]) -> *mut c_void {
    //libc!(libc::__tls_get_addr(p));
    let [module, offset] = *p;
    // Offset 0 is the generation field, and we don't support dynamic linking,
    // so we should only sever see 1 here.
    assert_eq!(module, 1);
    origin::current_thread_tls_addr(offset)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "C" fn ___tls_get_addr() {
    //libc!(libc::___tls_get_addr());
    unimplemented!("___tls_get_addr")
}