mod key;

use alloc::boxed::Box;
use core::convert::TryInto;
use core::ffi::c_void;
use core::mem::ManuallyDrop;
use core::ptr::{self, null_mut};
use core::sync::atomic::Ordering::SeqCst;
use core::sync::atomic::{AtomicBool, AtomicU32};
use core::time::Duration;
use origin::lock_api::{self, RawMutex as _, RawReentrantMutex, RawRwLock as _};
use origin::sync::{Condvar, RawMutex, RawRwLock};
use origin::Thread;

use libc::c_int;

struct GetThreadId;

unsafe impl lock_api::GetThreadId for GetThreadId {
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
    #[cfg(any(target_arch = "aarch64", target_arch = "x86"))]
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
            #[cfg(any(target_arch = "aarch64", target_arch = "x86"))]
            pad4: 0,
            #[cfg(target_arch = "x86")]
            pad5: 0,
        }
    }
}

#[allow(non_camel_case_types)]
#[repr(C)]
union pthread_mutex_u {
    normal: ManuallyDrop<RawMutex>,
    reentrant: ManuallyDrop<RawReentrantMutex<RawMutex, GetThreadId>>,
}

#[allow(non_camel_case_types)]
#[repr(C)]
struct PthreadMutexT {
    kind: AtomicU32,
    u: pthread_mutex_u,
    pad0: usize,
    #[cfg(any(target_arch = "aarch64", target_arch = "x86"))]
    pad1: usize,
}
libc_type!(PthreadMutexT, pthread_mutex_t);

#[allow(non_camel_case_types)]
#[repr(C)]
struct PthreadRwlockT {
    lock: RawRwLock,
    exclusive: AtomicBool,
    pad0: usize,
    pad1: usize,
    pad2: usize,
    pad3: usize,
    pad4: usize,
}
libc_type!(PthreadRwlockT, pthread_rwlock_t);

#[allow(non_camel_case_types)]
#[repr(C)]
struct PthreadMutexattrT {
    kind: AtomicU32,
    #[cfg(target_arch = "aarch64")]
    pad0: u32,
}
libc_type!(PthreadMutexattrT, pthread_mutexattr_t);

#[allow(non_camel_case_types)]
#[cfg_attr(target_pointer_width = "32", repr(C, align(4)))]
#[cfg_attr(target_pointer_width = "64", repr(C, align(8)))]
struct PthreadRwlockattrT {
    kind: AtomicU32,
    pad0: u32,
}
libc_type!(PthreadRwlockattrT, pthread_rwlockattr_t);

#[no_mangle]
unsafe extern "C" fn pthread_self() -> PthreadT {
    libc!(ptr::from_exposed_addr_mut(libc::pthread_self() as _));
    origin::current_thread().to_raw().cast()
}

#[no_mangle]
unsafe extern "C" fn pthread_getattr_np(thread: PthreadT, attr: *mut PthreadAttrT) -> c_int {
    libc!(libc::pthread_getattr_np(
        thread.expose_addr() as _,
        checked_cast!(attr)
    ));
    let (stack_addr, stack_size, guard_size) =
        origin::thread_stack(Thread::from_raw(thread.cast()));
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
            #[cfg(any(target_arch = "aarch64", target_arch = "x86"))]
            pad4: 0,
            #[cfg(target_arch = "x86")]
            pad5: 0,
        },
    );
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_attr_init(attr: *mut PthreadAttrT) -> c_int {
    libc!(libc::pthread_attr_init(checked_cast!(attr)));
    ptr::write(attr, PthreadAttrT::default());
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_attr_destroy(attr: *mut PthreadAttrT) -> c_int {
    libc!(libc::pthread_attr_destroy(checked_cast!(attr)));
    ptr::drop_in_place(attr);
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_attr_getstack(
    attr: *const PthreadAttrT,
    stackaddr: *mut *mut c_void,
    stacksize: *mut usize,
) -> c_int {
    libc!(libc::pthread_attr_getstack(
        checked_cast!(attr),
        stackaddr,
        stacksize
    ));
    *stackaddr = (*attr).stack_addr;
    *stacksize = (*attr).stack_size;
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_mutexattr_destroy(attr: *mut PthreadMutexattrT) -> c_int {
    libc!(libc::pthread_mutexattr_destroy(checked_cast!(attr)));
    ptr::drop_in_place(attr);
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_mutexattr_init(attr: *mut PthreadMutexattrT) -> c_int {
    libc!(libc::pthread_mutexattr_init(checked_cast!(attr)));
    ptr::write(
        attr,
        PthreadMutexattrT {
            kind: AtomicU32::new(libc::PTHREAD_MUTEX_DEFAULT as u32),
            #[cfg(target_arch = "aarch64")]
            pad0: 0,
        },
    );
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_mutexattr_settype(attr: *mut PthreadMutexattrT, kind: c_int) -> c_int {
    libc!(libc::pthread_mutexattr_settype(checked_cast!(attr), kind));
    (*attr).kind = AtomicU32::new(kind as u32);
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_mutex_destroy(mutex: *mut PthreadMutexT) -> c_int {
    libc!(libc::pthread_mutex_destroy(checked_cast!(mutex)));
    match (*mutex).kind.load(SeqCst) as i32 {
        libc::PTHREAD_MUTEX_NORMAL => ManuallyDrop::drop(&mut (*mutex).u.normal),
        libc::PTHREAD_MUTEX_RECURSIVE => ManuallyDrop::drop(&mut (*mutex).u.reentrant),
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
    libc!(libc::pthread_mutex_init(
        checked_cast!(mutex),
        checked_cast!(mutexattr)
    ));
    let kind = if mutexattr.is_null() {
        libc::PTHREAD_MUTEX_DEFAULT as u32
    } else {
        (*mutexattr).kind.load(SeqCst)
    };

    match kind as i32 {
        libc::PTHREAD_MUTEX_NORMAL => {
            ptr::write(&mut (*mutex).u.normal, ManuallyDrop::new(RawMutex::INIT))
        }
        libc::PTHREAD_MUTEX_RECURSIVE => ptr::write(
            &mut (*mutex).u.reentrant,
            ManuallyDrop::new(RawReentrantMutex::INIT),
        ),
        libc::PTHREAD_MUTEX_ERRORCHECK => unimplemented!("PTHREAD_MUTEX_ERRORCHECK"),
        other => unimplemented!("unsupported pthread mutex kind {}", other),
    }
    (*mutex).kind.store(kind, SeqCst);
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_mutex_lock(mutex: *mut PthreadMutexT) -> c_int {
    libc!(libc::pthread_mutex_lock(checked_cast!(mutex)));
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
    libc!(libc::pthread_mutex_trylock(checked_cast!(mutex)));
    if match (*mutex).kind.load(SeqCst) as i32 {
        libc::PTHREAD_MUTEX_NORMAL => (*mutex).u.normal.try_lock(),
        libc::PTHREAD_MUTEX_RECURSIVE => (*mutex).u.reentrant.try_lock(),
        libc::PTHREAD_MUTEX_ERRORCHECK => unimplemented!("PTHREAD_MUTEX_ERRORCHECK"),
        other => unimplemented!("unsupported pthread mutex kind {}", other),
    } {
        0
    } else {
        rustix::io::Errno::BUSY.raw_os_error()
    }
}

#[no_mangle]
unsafe extern "C" fn pthread_mutex_unlock(mutex: *mut PthreadMutexT) -> c_int {
    libc!(libc::pthread_mutex_unlock(checked_cast!(mutex)));
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
        rustix::io::Errno::BUSY.raw_os_error()
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
        rustix::io::Errno::BUSY.raw_os_error()
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
unsafe extern "C" fn pthread_attr_getguardsize(
    attr: *const PthreadAttrT,
    guardsize: *mut usize,
) -> c_int {
    libc!(libc::pthread_attr_getguardsize(
        checked_cast!(attr),
        guardsize
    ));
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
    inner: Condvar,
    attr: PthreadCondattrT,
    pad: [u8; SIZEOF_PTHREAD_COND_T
        - core::mem::size_of::<Condvar>()
        - core::mem::size_of::<PthreadCondattrT>()],
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

impl Default for PthreadCondattrT {
    fn default() -> Self {
        Self {
            pad: [0_u8; SIZEOF_PTHREAD_CONDATTR_T],
        }
    }
}

libc_type!(PthreadCondattrT, pthread_condattr_t);

#[no_mangle]
unsafe extern "C" fn pthread_condattr_destroy(attr: *mut PthreadCondattrT) -> c_int {
    libc!(libc::pthread_condattr_destroy(checked_cast!(attr)));
    ptr::drop_in_place(attr);
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_condattr_init(attr: *mut PthreadCondattrT) -> c_int {
    libc!(libc::pthread_condattr_init(checked_cast!(attr)));
    ptr::write(attr, PthreadCondattrT::default());
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
        rustix::io::stderr(),
        b"unimplemented: pthread_condattr_setclock\n",
    )
    .ok();
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_cond_broadcast(cond: *mut PthreadCondT) -> c_int {
    libc!(libc::pthread_cond_broadcast(checked_cast!(cond)));
    (*cond).inner.notify_all();
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_cond_destroy(cond: *mut PthreadCondT) -> c_int {
    libc!(libc::pthread_cond_destroy(checked_cast!(cond)));
    ptr::drop_in_place(cond);
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
    ptr::write(
        cond,
        PthreadCondT {
            inner: Condvar::new(),
            attr: ptr::read(attr),
            pad: [0_u8;
                SIZEOF_PTHREAD_COND_T
                    - core::mem::size_of::<Condvar>()
                    - core::mem::size_of::<PthreadCondattrT>()],
        },
    );
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_cond_signal(cond: *mut PthreadCondT) -> c_int {
    libc!(libc::pthread_cond_signal(checked_cast!(cond)));
    (*cond).inner.notify_one();
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_cond_wait(cond: *mut PthreadCondT, lock: *mut PthreadMutexT) -> c_int {
    libc!(libc::pthread_cond_wait(
        checked_cast!(cond),
        checked_cast!(lock)
    ));
    match (*lock).kind.load(SeqCst) as i32 {
        libc::PTHREAD_MUTEX_NORMAL => (*cond).inner.wait(&(*lock).u.normal),
        libc::PTHREAD_MUTEX_RECURSIVE => unimplemented!("PTHREAD_MUTEX_RECURSIVE"),
        libc::PTHREAD_MUTEX_ERRORCHECK => unimplemented!("PTHREAD_MUTEX_ERRORCHECK"),
        other => unimplemented!("unsupported pthread mutex kind {}", other),
    }
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_cond_timedwait(
    cond: *mut PthreadCondT,
    lock: *mut PthreadMutexT,
    abstime: *const libc::timespec,
) -> c_int {
    libc!(libc::pthread_cond_timedwait(
        checked_cast!(cond),
        checked_cast!(lock),
        abstime,
    ));
    let abstime = ptr::read(abstime);
    let duration = Duration::new(
        abstime.tv_sec.try_into().unwrap(),
        abstime.tv_nsec.try_into().unwrap(),
    );
    match (*lock).kind.load(SeqCst) as i32 {
        libc::PTHREAD_MUTEX_NORMAL => {
            if (*cond).inner.wait_timeout(&(*lock).u.normal, duration) {
                0
            } else {
                libc::ETIMEDOUT
            }
        }
        libc::PTHREAD_MUTEX_RECURSIVE => unimplemented!("PTHREAD_MUTEX_RECURSIVE"),
        libc::PTHREAD_MUTEX_ERRORCHECK => unimplemented!("PTHREAD_MUTEX_ERRORCHECK"),
        other => unimplemented!("unsupported pthread mutex kind {}", other),
    }
}

#[no_mangle]
unsafe extern "C" fn pthread_rwlock_init(
    rwlock: *mut PthreadRwlockT,
    rwlockattr: *const PthreadRwlockattrT,
) -> c_int {
    libc!(libc::pthread_rwlock_init(
        checked_cast!(rwlock),
        checked_cast!(rwlockattr)
    ));
    ptr::write(&mut (*rwlock).lock, RawRwLock::INIT);
    (*rwlock).exclusive.store(false, SeqCst);

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
    libc!(libc::pthread_create(
        pthread as _,
        checked_cast!(attr),
        fn_,
        arg
    ));
    let PthreadAttrT {
        stack_addr,
        stack_size,
        guard_size,
        pad0: _,
        pad1: _,
        pad2: _,
        pad3: _,
        #[cfg(any(target_arch = "aarch64", target_arch = "x86"))]
            pad4: _,
        #[cfg(target_arch = "x86")]
            pad5: _,
    } = if attr.is_null() {
        PthreadAttrT::default()
    } else {
        ptr::read(attr)
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

    pthread.write(thread.to_raw().cast());
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_detach(pthread: PthreadT) -> c_int {
    libc!(libc::pthread_detach(pthread.expose_addr() as _));
    origin::detach_thread(Thread::from_raw(pthread.cast()));
    0
}

#[no_mangle]
unsafe extern "C" fn pthread_join(pthread: PthreadT, retval: *mut *mut c_void) -> c_int {
    libc!(libc::pthread_join(pthread.expose_addr() as _, retval));
    assert!(
        retval.is_null(),
        "pthread_join return values not supported yet"
    );

    origin::join_thread(Thread::from_raw(pthread.cast()));
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
    libc!(libc::pthread_attr_setstacksize(
        checked_cast!(attr),
        stacksize
    ));
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

// TODO: The ordering that we need here is
// all the pthread_cleanup functions,
// then all thread-local dtors,
// and then finally all static
// (`__cxa_thread_atexit_impl`) dtors.
//
// We don't have the mechanism for that yet though.
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

// TODO: See comment on `pthread_clean_push` about the
// ordering gurantees that programs expect.
#[no_mangle]
unsafe extern "C" fn __cxa_thread_atexit_impl(
    func: unsafe extern "C" fn(*mut c_void),
    obj: *mut c_void,
    _dso_symbol: *mut c_void,
) -> c_int {
    // TODO: libc!(libc::__cxa_thread_atexit_impl(func, obj, _dso_symbol));
    origin::at_thread_exit(Box::new(move || func(obj)));
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
