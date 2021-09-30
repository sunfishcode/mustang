//! Pthread functionality is stubbed out. This is sort of just enough for
//! programs that don't create new threads.

#![allow(non_camel_case_types)]

use crate::data;
use crate::raw_mutex::RawMutex;
use origin::Thread;
use parking_lot::lock_api::RawRwLock;
use std::convert::TryInto;
use std::ffi::c_void;
use std::os::raw::{c_int, c_ulong};
use std::ptr::{self, null, null_mut};
use std::sync::atomic::Ordering::SeqCst;
use std::sync::atomic::{AtomicBool, AtomicU32};

struct GetThreadId;

unsafe impl parking_lot::lock_api::GetThreadId for GetThreadId {
    const INIT: Self = Self;

    fn nonzero_thread_id(&self) -> std::num::NonZeroUsize {
        (origin::current_thread_id().as_raw() as usize)
            .try_into()
            .unwrap()
    }
}

#[allow(non_camel_case_types)]
type pthread_t = c_ulong;
libc_type!(pthread_t, pthread_t);

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Clone)]
struct pthread_attr_t {
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
libc_type!(pthread_attr_t, pthread_attr_t);

impl Default for pthread_attr_t {
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
    // we use `wee_alloc` for global allocations, it uses a mutex, and
    // `parking_lot`'s `RawMutex` performs global allocations.
    normal: RawMutex,
    reentrant: parking_lot::lock_api::RawReentrantMutex<parking_lot::RawMutex, GetThreadId>,
}

#[allow(non_camel_case_types)]
#[repr(C)]
struct pthread_mutex_t {
    kind: AtomicU32,
    u: pthread_mutex_u,
    pad0: usize,
    #[cfg(target_arch = "x86")]
    pad1: usize,
}
libc_type!(pthread_mutex_t, pthread_mutex_t);

#[allow(non_camel_case_types)]
#[repr(C)]
struct pthread_rwlock_t {
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
libc_type!(pthread_rwlock_t, pthread_rwlock_t);

#[allow(non_camel_case_types)]
#[repr(C)]
struct pthread_mutexattr_t {
    kind: AtomicU32,
}
libc_type!(pthread_mutexattr_t, pthread_mutexattr_t);

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_self() -> pthread_t {
    libc!(pthread_self());
    origin::current_thread() as pthread_t
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_getattr_np(thread: pthread_t, attr: *mut pthread_attr_t) -> c_int {
    libc!(pthread_getattr_np(thread, same_ptr_mut(attr)));
    let (stack_addr, stack_size, guard_size) = Thread::stack(thread as *mut Thread);
    ptr::write(
        attr,
        pthread_attr_t {
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

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_attr_init(attr: *mut pthread_attr_t) -> c_int {
    libc!(pthread_attr_init(same_ptr_mut(attr)));
    ptr::write(attr, pthread_attr_t::default());
    0
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_attr_destroy(attr: *mut pthread_attr_t) -> c_int {
    libc!(pthread_attr_destroy(same_ptr_mut(attr)));
    ptr::drop_in_place(attr);
    0
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_attr_getstack(
    attr: *const pthread_attr_t,
    stackaddr: *mut *mut c_void,
    stacksize: *mut usize,
) -> c_int {
    libc!(pthread_attr_getstack(same_ptr(attr), stackaddr, stacksize));
    *stackaddr = (*attr).stack_addr;
    *stacksize = (*attr).stack_size;
    0
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_getspecific() -> *const c_void {
    //libc!(pthread_getspecific());
    rsix::io::write(&rsix::io::stderr(), b"unimplemented: pthread_getspecific\n").ok();
    null()
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_key_create() -> c_int {
    //libc!(pthread_key_create());
    rsix::io::write(&rsix::io::stderr(), b"unimplemented: pthread_key_create\n").ok();
    0
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_key_delete() -> c_int {
    //libc!(pthread_key_delete());
    rsix::io::write(&rsix::io::stderr(), b"unimplemented: pthread_key_delete\n").ok();
    0
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_mutexattr_destroy(attr: *mut pthread_mutexattr_t) -> c_int {
    libc!(pthread_mutexattr_destroy(same_ptr_mut(attr)));
    ptr::drop_in_place(attr);
    0
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_mutexattr_init(attr: *mut pthread_mutexattr_t) -> c_int {
    libc!(pthread_mutexattr_init(same_ptr_mut(attr)));
    ptr::write(
        attr,
        pthread_mutexattr_t {
            kind: AtomicU32::new(data::PTHREAD_MUTEX_DEFAULT as u32),
        },
    );
    0
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_mutexattr_settype(
    attr: *mut pthread_mutexattr_t,
    kind: c_int,
) -> c_int {
    libc!(pthread_mutexattr_settype(same_ptr_mut(attr), kind));
    (*attr).kind = AtomicU32::new(kind as u32);
    0
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_mutex_destroy(mutex: *mut pthread_mutex_t) -> c_int {
    libc!(pthread_mutex_destroy(same_ptr_mut(mutex)));
    match (*mutex).kind.load(SeqCst) as i32 {
        data::PTHREAD_MUTEX_NORMAL => ptr::drop_in_place(&mut (*mutex).u.normal),
        data::PTHREAD_MUTEX_RECURSIVE => ptr::drop_in_place(&mut (*mutex).u.reentrant),
        data::PTHREAD_MUTEX_ERRORCHECK => unimplemented!("PTHREAD_MUTEX_ERRORCHECK"),
        other => unimplemented!("unsupported pthread mutex kind {}", other),
    }
    (*mutex).kind.store(!0, SeqCst);
    0
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_mutex_init(
    mutex: *mut pthread_mutex_t,
    mutexattr: *const pthread_mutexattr_t,
) -> c_int {
    libc!(pthread_mutex_init(same_ptr_mut(mutex), same_ptr(mutexattr)));
    let kind = (*mutexattr).kind.load(SeqCst);
    match kind as i32 {
        data::PTHREAD_MUTEX_NORMAL => ptr::write(&mut (*mutex).u.normal, RawMutex::new()),
        data::PTHREAD_MUTEX_RECURSIVE => ptr::write(
            &mut (*mutex).u.reentrant,
            parking_lot::lock_api::RawReentrantMutex::<parking_lot::RawMutex, GetThreadId>::INIT,
        ),
        data::PTHREAD_MUTEX_ERRORCHECK => unimplemented!("PTHREAD_MUTEX_ERRORCHECK"),
        other => unimplemented!("unsupported pthread mutex kind {}", other),
    }
    (*mutex).kind.store(kind, SeqCst);
    0
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_mutex_lock(mutex: *mut pthread_mutex_t) -> c_int {
    libc!(pthread_mutex_lock(same_ptr_mut(mutex)));
    match (*mutex).kind.load(SeqCst) as i32 {
        data::PTHREAD_MUTEX_NORMAL => (*mutex).u.normal.lock(),
        data::PTHREAD_MUTEX_RECURSIVE => (*mutex).u.reentrant.lock(),
        data::PTHREAD_MUTEX_ERRORCHECK => unimplemented!("PTHREAD_MUTEX_ERRORCHECK"),
        other => unimplemented!("unsupported pthread mutex kind {}", other),
    }
    0
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_mutex_trylock(mutex: *mut pthread_mutex_t) -> c_int {
    libc!(pthread_mutex_trylock(same_ptr_mut(mutex)));
    if match (*mutex).kind.load(SeqCst) as i32 {
        data::PTHREAD_MUTEX_NORMAL => (*mutex).u.normal.try_lock(),
        data::PTHREAD_MUTEX_RECURSIVE => (*mutex).u.reentrant.try_lock(),
        data::PTHREAD_MUTEX_ERRORCHECK => unimplemented!("PTHREAD_MUTEX_ERRORCHECK"),
        other => unimplemented!("unsupported pthread mutex kind {}", other),
    } {
        0
    } else {
        rsix::io::Error::BUSY.raw_os_error()
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_mutex_unlock(mutex: *mut pthread_mutex_t) -> c_int {
    libc!(pthread_mutex_unlock(same_ptr_mut(mutex)));
    match (*mutex).kind.load(SeqCst) as i32 {
        data::PTHREAD_MUTEX_NORMAL => (*mutex).u.normal.unlock(),
        data::PTHREAD_MUTEX_RECURSIVE => (*mutex).u.reentrant.unlock(),
        data::PTHREAD_MUTEX_ERRORCHECK => unimplemented!("PTHREAD_MUTEX_ERRORCHECK"),
        other => unimplemented!("unsupported pthread mutex kind {}", other),
    }
    0
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_rwlock_rdlock(rwlock: *mut pthread_rwlock_t) -> c_int {
    libc!(pthread_rwlock_rdlock(same_ptr_mut(rwlock)));
    (*rwlock).lock.lock_shared();
    (*rwlock).exclusive.store(false, SeqCst);
    0
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_rwlock_unlock(rwlock: *mut pthread_rwlock_t) -> c_int {
    libc!(pthread_rwlock_unlock(same_ptr_mut(rwlock)));
    if (*rwlock).exclusive.load(SeqCst) {
        (*rwlock).lock.lock_exclusive();
    } else {
        (*rwlock).lock.lock_shared();
    }
    0
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_setspecific() -> c_int {
    //libc!(pthread_setspecific());
    rsix::io::write(&rsix::io::stderr(), b"unimplemented: pthread_getspecific\n").ok();
    0
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_attr_getguardsize(
    attr: *const pthread_attr_t,
    guardsize: *mut usize,
) -> c_int {
    libc!(pthread_attr_getguardsize(same_ptr(attr), guardsize));
    *guardsize = (*attr).guard_size;
    0
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_attr_setguardsize(
    attr: *mut pthread_attr_t,
    guardsize: usize,
) -> c_int {
    // TODO: libc!(pthread_attr_setguardsize(attr, guardsize));
    (*attr).guard_size = guardsize;
    0
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_condattr_destroy() -> c_int {
    //libc!(pthread_condattr_destroy());
    rsix::io::write(
        &rsix::io::stderr(),
        b"unimplemented: pthread_condattr_destroy\n",
    )
    .ok();
    0
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_condattr_init() -> c_int {
    //libc!(pthread_condattr_init());
    rsix::io::write(
        &rsix::io::stderr(),
        b"unimplemented: pthread_condattr_init\n",
    )
    .ok();
    0
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_condattr_setclock() -> c_int {
    //libc!(pthread_condattr_setclock());
    rsix::io::write(
        &rsix::io::stderr(),
        b"unimplemented: pthread_condattr_setclock\n",
    )
    .ok();
    0
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_cond_broadcast() -> c_int {
    //libc!(pthread_cond_broadcast());
    rsix::io::write(
        &rsix::io::stderr(),
        b"unimplemented: pthread_cond_broadcast\n",
    )
    .ok();
    0
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_cond_destroy() -> c_int {
    //libc!(pthread_cond_destroy());
    rsix::io::write(
        &rsix::io::stderr(),
        b"unimplemented: pthread_cond_destroy\n",
    )
    .ok();
    0
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_cond_init() -> c_int {
    //libc!(pthread_cond_init());
    rsix::io::write(&rsix::io::stderr(), b"unimplemented: pthread_cond_init\n").ok();
    0
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_cond_signal() -> c_int {
    //libc!(pthread_cond_signal());
    rsix::io::write(&rsix::io::stderr(), b"unimplemented: pthread_cond_signal\n").ok();
    0
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_cond_wait() -> c_int {
    //libc!(pthread_cond_wait());
    rsix::io::write(&rsix::io::stderr(), b"unimplemented: pthread_cond_wait\n").ok();
    0
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_cond_timedwait() {
    //libc!(pthread_cond_timedwait());
    rsix::io::write(
        &rsix::io::stderr(),
        b"unimplemented: pthread_cond_timedwait\n",
    )
    .ok();
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_rwlock_destroy(rwlock: *mut pthread_rwlock_t) -> c_int {
    libc!(pthread_rwlock_destroy(same_ptr_mut(rwlock)));
    ptr::drop_in_place(rwlock);
    0
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_rwlock_wrlock(rwlock: *mut pthread_rwlock_t) -> c_int {
    libc!(pthread_rwlock_wrlock(same_ptr_mut(rwlock)));
    (*rwlock).lock.lock_exclusive();
    (*rwlock).exclusive.store(true, SeqCst);
    0
}

// TODO: signals, create thread-local storage, arrange for thread-local storage
// destructors to run, free the `mmap`.
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_create(
    pthread: *mut pthread_t,
    attr: *const pthread_attr_t,
    fn_: extern "C" fn(*mut c_void) -> *mut c_void,
    arg: *mut c_void,
) -> c_int {
    libc!(pthread_create(pthread, same_ptr(attr), fn_, arg));
    let pthread_attr_t {
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
        pthread_attr_t::default()
    } else {
        (*attr).clone()
    };
    assert!(stack_addr.is_null());

    let thread_data = match origin::create_thread(fn_, arg, stack_size, guard_size) {
        Ok(thread_data) => thread_data,
        Err(e) => return e.raw_os_error(),
    };

    pthread.write(thread_data as pthread_t);
    0
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_detach(pthread: pthread_t) -> c_int {
    libc!(pthread_detach(pthread));
    origin::detach_thread(pthread as *mut Thread);
    0
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_join(pthread: pthread_t, retval: *mut *mut c_void) -> c_int {
    libc!(pthread_join(pthread, retval));
    assert!(
        retval.is_null(),
        "pthread_join return values not supported yet"
    );

    match origin::join_thread(pthread as *mut Thread) {
        Ok(()) => 0,
        Err(err) => err.raw_os_error(),
    }
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_sigmask() -> c_int {
    //libc!(pthread_sigmask());
    rsix::io::write(&rsix::io::stderr(), b"unimplemented: pthread_sigmask\n").ok();
    0
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_attr_setstacksize(
    attr: *mut pthread_attr_t,
    stacksize: usize,
) -> c_int {
    libc!(pthread_attr_setstacksize(same_ptr_mut(attr), stacksize));
    (*attr).stack_size = stacksize;
    0
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_cancel() -> c_int {
    //libc!(pthread_cancel());
    // `pthread_cancel` may be tricky to implement, because it seems glibc's
    // cancellation mechanism uses `setjmp` to a `jmp_buf` store in
    // `__libc_start_main`'s stack, and the `initialize-c-runtime` crate
    // itself `longjmp`s out of `__libc_start_main`.
    //
    // <https://github.com/sunfishcode/mustang/pull/4#issuecomment-915872029>
    unimplemented!("pthread_cancel")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_exit() -> c_int {
    //libc!(pthread_exit());
    // As with `pthread_cancel`, `pthread_exit` may be tricky to implement.
    unimplemented!("pthread_exit")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_cleanup_push() -> c_int {
    //libc!(pthread_cleanup_push());
    unimplemented!("pthread_cleanup_push")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_cleanup_pop() -> c_int {
    //libc!(pthread_cleanup_pop());
    unimplemented!("pthread_cleanup_pop")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_setcancelstate() -> c_int {
    //libc!(pthread_setcancelstate());
    unimplemented!("pthread_setcancelstate")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_setcanceltype() -> c_int {
    //libc!(pthread_setcanceltype());
    unimplemented!("pthread_setcanceltype")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_testcancel() -> c_int {
    //libc!(pthread_testcancel());
    unimplemented!("pthread_testcancel")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn pthread_atfork(
    _prepare: unsafe extern "C" fn(),
    _parent: unsafe extern "C" fn(),
    _child: unsafe extern "C" fn(),
) -> c_int {
    libc!(pthread_atfork(_prepare, _parent, _child));

    // We don't implement `fork` yet, so there's nothing to do.

    0
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn __cxa_thread_atexit_impl(
    func: unsafe extern "C" fn(*mut c_void),
    obj: *mut c_void,
    _dso_symbol: *mut c_void,
) -> c_int {
    // TODO: libc!(__cxa_thread_atexit_impl(func, obj, _dso_symbol));
    origin::at_thread_exit(func, obj);
    0
}

/// Ensure that this module is linked in.
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
#[cold]
unsafe extern "C" fn __mustang_c_scape__pthread() {}
