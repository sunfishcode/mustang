//! Threads runtime implemented using rustix, asm, and raw Linux syscalls.
//!
//! This implementation is only used on mustang targets. See
//! threads_via_pthreads.rs for the non-mustang implementation.

use crate::arch::{
    clone, get_thread_pointer, munmap_and_exit_thread, set_thread_pointer, TLS_OFFSET,
};
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::any::Any;
use core::cmp::max;
use core::ffi::c_void;
use core::mem::{align_of, size_of};
use core::ptr::{self, drop_in_place, null, null_mut};
use core::slice;
use core::sync::atomic::Ordering::SeqCst;
use core::sync::atomic::{AtomicI32, AtomicU8};
use memoffset::offset_of;
use rustix::io;
use rustix::param::{linux_execfn, page_size};
use rustix::process::{getrlimit, Pid, Resource};
use rustix::runtime::{set_tid_address, StartupTlsInfo};
use rustix::thread::gettid;

/// The entrypoint where Rust code is first executed on a new thread.
///
/// This calls `fn_` on the new thread. When `fn_` returns, the thread exits.
///
/// # Safety
///
/// After calling `fn_`, this terminates the thread.
pub(super) unsafe extern "C" fn entry(fn_: *mut Box<dyn FnOnce() -> Option<Box<dyn Any>>>) -> ! {
    let fn_ = Box::from_raw(fn_);

    #[cfg(feature = "log")]
    log::trace!("Thread[{:?}] launched", current_thread_id());

    // Do some basic precondition checks, to ensure that our assembly code did
    // what we expect it to do. These are debug-only for now, to keep the
    // release-mode startup code simple to disassemble and inspect, while we're
    // getting started.
    #[cfg(debug_assertions)]
    {
        extern "C" {
            #[link_name = "llvm.frameaddress"]
            fn builtin_frame_address(level: i32) -> *const u8;
            #[link_name = "llvm.returnaddress"]
            fn builtin_return_address(level: i32) -> *const u8;
            #[cfg(target_arch = "aarch64")]
            #[link_name = "llvm.sponentry"]
            fn builtin_sponentry() -> *const u8;
        }

        // Check that the incoming stack pointer is where we expect it to be.
        debug_assert_eq!(builtin_return_address(0), core::ptr::null());
        debug_assert_ne!(builtin_frame_address(0), core::ptr::null());
        #[cfg(not(any(target_arch = "x86", target_arch = "arm")))]
        debug_assert_eq!(builtin_frame_address(0).addr() & 0xf, 0);
        #[cfg(target_arch = "arm")]
        debug_assert_eq!(builtin_frame_address(0).addr() & 0x3, 0);
        #[cfg(target_arch = "x86")]
        debug_assert_eq!(builtin_frame_address(0).addr() & 0xf, 8);
        debug_assert_eq!(builtin_frame_address(1), core::ptr::null());
        #[cfg(target_arch = "aarch64")]
        debug_assert_ne!(builtin_sponentry(), core::ptr::null());
        #[cfg(target_arch = "aarch64")]
        debug_assert_eq!(builtin_sponentry().addr() & 0xf, 0);

        // Check that `clone` stored our thread id as we expected.
        debug_assert_eq!(current_thread_id(), gettid());
    }

    // Call the user thread function. In `std`, this is `thread_start`. Ignore
    // the return value for now, as `std` doesn't need it.
    let _result = fn_();

    exit_thread()
}

/// Metadata describing a thread.
#[repr(C)]
struct Metadata {
    /// Crate-internal fields. On platforms where TLS data goes after the
    /// ABI-exposed fields, we store our fields before them.
    #[cfg(any(target_arch = "aarch64", target_arch = "arm", target_arch = "riscv64"))]
    thread: ThreadData,

    /// ABI-exposed fields. This is allocated at a platform-specific offset
    /// from the platform thread-pointer register value.
    abi: Abi,

    /// Crate-internal fields. On platforms where TLS data goes before the
    /// ABI-exposed fields, we store our fields after them.
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    thread: ThreadData,
}

/// Fields which accessed by user code via known offsets from the platform
/// thread-pointer register.
#[repr(C)]
#[cfg_attr(target_arch = "arm", repr(align(8)))]
struct Abi {
    /// Aarch64 has an ABI-exposed `dtv` field (though we don't yet implement
    /// dynamic linking).
    #[cfg(any(target_arch = "aarch64", target_arch = "arm"))]
    dtv: *const c_void,

    /// x86 and x86-64 put a copy of the thread-pointer register at the memory
    /// location pointed to by the thread-pointer register, because reading the
    /// thread-pointer register directly is slow.
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    this: *mut Abi,

    /// Padding to put the TLS data which follows at its known offset.
    #[cfg(any(target_arch = "aarch64", target_arch = "arm"))]
    pad: [usize; 1],
}

/// An opaque pointer to a thread.
#[derive(Copy, Clone)]
pub struct Thread(*mut ThreadData);

impl Thread {
    /// Convert to `Self` from a raw pointer.
    #[inline]
    pub fn from_raw(raw: *mut c_void) -> Self {
        Self(raw.cast())
    }

    /// Convert to a raw pointer from a `Self`.
    #[inline]
    pub fn to_raw(self) -> *mut c_void {
        self.0.cast()
    }
}

/// Data associated with a thread. This is not `repr(C)` and not ABI-exposed.
struct ThreadData {
    thread_id: AtomicI32,
    detached: AtomicU8,
    stack_addr: *mut c_void,
    stack_size: usize,
    guard_size: usize,
    map_size: usize,
    dtors: Vec<Box<dyn FnOnce()>>,
}

// Values for `ThreadData::detached`.
const INITIAL: u8 = 0;
const DETACHED: u8 = 1;
const ABANDONED: u8 = 2;

impl ThreadData {
    #[inline]
    fn new(
        tid: Option<Pid>,
        stack_addr: *mut c_void,
        stack_size: usize,
        guard_size: usize,
        map_size: usize,
    ) -> Self {
        Self {
            thread_id: AtomicI32::new(Pid::as_raw(tid)),
            detached: AtomicU8::new(INITIAL),
            stack_addr,
            stack_size,
            guard_size,
            map_size,
            dtors: Vec::new(),
        }
    }
}

#[inline]
fn current_metadata() -> *mut Metadata {
    get_thread_pointer()
        .cast::<u8>()
        .wrapping_sub(offset_of!(Metadata, abi))
        .cast()
}

/// Return a raw pointer to the data associated with the current thread.
#[inline]
pub fn current_thread() -> Thread {
    unsafe { Thread(&mut (*current_metadata()).thread) }
}

/// Return the current thread id.
///
/// This is the same as [`rustix::thread::gettid`], but loads the value from a
/// field in the runtime rather than making a system call.
#[inline]
pub fn current_thread_id() -> Pid {
    let raw = unsafe { (*current_thread().0).thread_id.load(SeqCst) };
    debug_assert!(raw > 0);
    let tid = unsafe { Pid::from_raw_unchecked(raw) };
    debug_assert_eq!(tid, gettid(), "`current_thread_id` disagrees with `gettid`");
    tid
}

/// Set the current thread id, after a `fork`.
///
/// The only valid use for this is in the implementation of libc-like `fork`
/// wrappers such as the one in c-scape. `posix_spawn`-like uses of `fork`
/// don't need to do this because they shouldn't do anything that cares about
/// the thread id before doing their `execve`.
///
/// # Safety
///
/// This must only be called immediately after a `fork` before any other
/// threads are created. `tid` must be the same value as what [`gettid`] would
/// return.
#[cfg(feature = "set_thread_id")]
#[doc(hidden)]
#[inline]
pub unsafe fn set_current_thread_id_after_a_fork(tid: Pid) {
    assert_ne!(
        tid.as_raw_nonzero().get(),
        (*current_thread().0).thread_id.load(SeqCst),
        "current thread ID already matches new thread ID"
    );
    assert_eq!(tid, gettid(), "new thread ID disagrees with `gettid`");
    (*current_thread().0)
        .thread_id
        .store(tid.as_raw_nonzero().get(), SeqCst);
}

/// Return the TLS entry for the current thread.
#[inline]
pub fn current_thread_tls_addr(offset: usize) -> *mut c_void {
    // Platforms where TLS data goes after the ABI-exposed fields.
    #[cfg(any(target_arch = "aarch64", target_arch = "arm", target_arch = "riscv64"))]
    {
        crate::arch::get_thread_pointer()
            .cast::<u8>()
            .wrapping_add(TLS_OFFSET)
            .wrapping_add(size_of::<Abi>())
            .wrapping_add(offset)
            .cast()
    }

    // Platforms where TLS data goes before the ABI-exposed fields.
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    unsafe {
        get_thread_pointer()
            .cast::<u8>()
            .wrapping_add(TLS_OFFSET)
            .wrapping_sub(STARTUP_TLS_INFO.mem_size)
            .wrapping_add(offset)
            .cast()
    }
}

/// Return the current thread's stack address (lowest address), size, and guard
/// size.
///
/// # Safety
///
/// `thread` must point to a valid and live thread record.
#[inline]
pub unsafe fn thread_stack(thread: Thread) -> (*mut c_void, usize, usize) {
    let data = &*thread.0;
    (data.stack_addr, data.stack_size, data.guard_size)
}

/// Registers a function to call when the current thread exits.
pub fn at_thread_exit(func: Box<dyn FnOnce()>) {
    // Safety: `current_thread()` points to thread-local data which is valid
    // as long as the thread is alive.
    unsafe {
        (*current_thread().0).dtors.push(func);
    }
}

/// Call the destructors registered with [`at_thread_exit`].
pub(crate) fn call_thread_dtors(current: Thread) {
    // Run the `dtors`, in reverse order of registration. Note that destructors
    // may register new destructors.
    //
    // Safety: `current` points to thread-local data which is valid as long
    // as the thread is alive.
    while let Some(func) = unsafe { (*current.0).dtors.pop() } {
        #[cfg(feature = "log")]
        if log::log_enabled!(log::Level::Trace) {
            log::trace!(
                "Thread[{:?}] calling `at_thread_exit`-registered function",
                unsafe { (*current.0).thread_id.load(SeqCst) },
            );
        }

        func();
    }
}

/// Call the destructors registered with [`at_thread_exit`] and exit the
/// thread.
unsafe fn exit_thread() -> ! {
    let current = current_thread();

    // Call functions registered with `at_thread_exit`.
    call_thread_dtors(current);

    // Read the thread's state, and set it to `ABANDONED` if it was `INITIAL`,
    // which tells `join_thread` to free the memory. Otherwise, it's in the
    // `DETACHED` state, and we free the memory immediately.
    let state = (*current.0)
        .detached
        .compare_exchange(INITIAL, ABANDONED, SeqCst, SeqCst);
    if let Err(e) = state {
        // The thread was detached. Prepare to free the memory. First read out
        // all the fields that we'll need before freeing it.
        #[cfg(feature = "log")]
        let current_thread_id = (*current.0).thread_id.load(SeqCst);
        let current_map_size = (*current.0).map_size;
        let current_stack_addr = (*current.0).stack_addr;
        let current_guard_size = (*current.0).guard_size;

        #[cfg(feature = "log")]
        log::trace!("Thread[{:?}] exiting as detached", current_thread_id);
        debug_assert_eq!(e, DETACHED);

        // Deallocate the `ThreadData`.
        drop_in_place(current.0);

        // Free the thread's `mmap` region, if we allocated it.
        let map_size = current_map_size;
        if map_size != 0 {
            // Null out the tid address so that the kernel doesn't write to
            // memory that we've freed trying to clear our tid when we exit.
            let _ = set_tid_address(null_mut());

            // `munmap` the memory, which also frees the stack we're currently
            // on, and do an `exit` carefully without touching the stack.
            let map = current_stack_addr.cast::<u8>().sub(current_guard_size);
            munmap_and_exit_thread(map.cast(), map_size);
        }
    } else {
        // The thread was not detached, so its memory will be freed when it's
        // joined.
        #[cfg(feature = "log")]
        if log::log_enabled!(log::Level::Trace) {
            log::trace!(
                "Thread[{:?}] exiting as joinable",
                (*current.0).thread_id.load(SeqCst)
            );
        }
    }

    // Terminate the thread.
    rustix::runtime::exit_thread(0)
}

/// Initialize the main thread.
///
/// This function is similar to `create_thread` except that the OS thread is
/// already created, and already has a stack (which we need to locate), and is
/// already running.
pub(super) unsafe fn initialize_main_thread(mem: *mut c_void) {
    use rustix::mm::{mmap_anonymous, MapFlags, ProtFlags};

    // Read the TLS information from the ELF header.
    STARTUP_TLS_INFO = rustix::runtime::startup_tls_info();

    // Determine the top of the stack. Linux puts the `AT_EXECFN` string at
    // the top, so find the end of that, and then round up to the page size.
    // See <https://lwn.net/Articles/631631/> for details.
    let execfn = linux_execfn().to_bytes_with_nul();
    let stack_base = execfn.as_ptr().add(execfn.len());
    let stack_base = stack_base.map_addr(|ptr| round_up(ptr, page_size())) as *mut c_void;

    // We're running before any user code, so the startup soft stack limit is
    // the effective stack size. And Linux doesn't set up a guard page for the
    // main thread.
    let stack_map_size = getrlimit(Resource::Stack).current.unwrap() as usize;
    let stack_least = stack_base.cast::<u8>().sub(stack_map_size);
    let stack_size = stack_least.offset_from(mem.cast::<u8>()) as usize;
    let guard_size = 0;
    let map_size = 0;

    // Compute relevant alignments.
    let tls_data_align = STARTUP_TLS_INFO.align;
    let header_align = align_of::<Metadata>();
    let metadata_align = max(tls_data_align, header_align);

    // Compute the size to allocate for thread data.
    let mut alloc_size = 0;

    // Variant II: TLS data goes below the TCB.
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    let tls_data_bottom = alloc_size;

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        alloc_size += round_up(STARTUP_TLS_INFO.mem_size, metadata_align);
    }

    let header = alloc_size;

    alloc_size += size_of::<Metadata>();

    // Variant I: TLS data goes above the TCB.
    #[cfg(any(target_arch = "aarch64", target_arch = "arm", target_arch = "riscv64"))]
    {
        alloc_size = round_up(alloc_size, tls_data_align);
    }

    #[cfg(any(target_arch = "aarch64", target_arch = "arm", target_arch = "riscv64"))]
    let tls_data_bottom = alloc_size;

    #[cfg(any(target_arch = "aarch64", target_arch = "arm", target_arch = "riscv64"))]
    {
        alloc_size += round_up(STARTUP_TLS_INFO.mem_size, tls_data_align);
    }

    // Allocate the thread data. Use `mmap_anonymous` rather than `alloc` here
    // as the allocator may depend on thread-local data, which is what we're
    // initializing here.
    let new = mmap_anonymous(
        null_mut(),
        alloc_size,
        ProtFlags::READ | ProtFlags::WRITE,
        MapFlags::PRIVATE,
    )
    .unwrap()
    .cast::<u8>();
    debug_assert_eq!(new.addr() % metadata_align, 0);

    let tls_data = new.add(tls_data_bottom);
    let metadata: *mut Metadata = new.add(header).cast();
    let newtls: *mut Abi = &mut (*metadata).abi;

    let thread_id_ptr = (*metadata).thread.thread_id.as_ptr();
    let tid = rustix::runtime::set_tid_address(thread_id_ptr.cast());

    // Initialize the thread metadata.
    ptr::write(
        metadata,
        Metadata {
            abi: Abi {
                #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
                this: newtls,
                #[cfg(any(target_arch = "aarch64", target_arch = "arm"))]
                dtv: null(),
                #[cfg(any(target_arch = "aarch64", target_arch = "arm"))]
                pad: [0_usize; 1],
            },
            thread: ThreadData::new(
                Some(tid),
                stack_least.cast(),
                stack_size,
                guard_size,
                map_size,
            ),
        },
    );

    // Initialize the TLS data with explicit initializer data.
    slice::from_raw_parts_mut(tls_data, STARTUP_TLS_INFO.file_size).copy_from_slice(
        slice::from_raw_parts(
            STARTUP_TLS_INFO.addr.cast::<u8>(),
            STARTUP_TLS_INFO.file_size,
        ),
    );

    // Initialize the TLS data beyond `file_size` which is zero-filled.
    slice::from_raw_parts_mut(
        tls_data.add(STARTUP_TLS_INFO.file_size),
        STARTUP_TLS_INFO.mem_size - STARTUP_TLS_INFO.file_size,
    )
    .fill(0);

    // Point the platform thread-pointer register at the new thread metadata.
    set_thread_pointer(newtls.cast::<u8>().cast());
}

/// Creates a new thread.
///
/// `fn_` is called on the new thread.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn create_thread(
    fn_: Box<dyn FnOnce() -> Option<Box<dyn Any>>>,
    stack_size: usize,
    guard_size: usize,
) -> io::Result<Thread> {
    use rustix::mm::{mmap_anonymous, mprotect, MapFlags, MprotectFlags, ProtFlags};

    // Safety: `STARTUP_TLS_INFO` is initialized at program startup before
    // we come here creating new threads.
    let (startup_tls_align, startup_tls_mem_size) =
        unsafe { (STARTUP_TLS_INFO.align, STARTUP_TLS_INFO.mem_size) };

    // Compute relevant alignments.
    let tls_data_align = startup_tls_align;
    let page_align = page_size();
    let stack_align = 16;
    let header_align = align_of::<Metadata>();
    let metadata_align = max(tls_data_align, header_align);
    let stack_metadata_align = max(stack_align, metadata_align);
    assert!(stack_metadata_align <= page_align);

    // Compute the `mmap` size.
    let mut map_size = 0;

    map_size += round_up(guard_size, page_align);

    let stack_bottom = map_size;

    map_size += round_up(stack_size, stack_metadata_align);

    let stack_top = map_size;

    // Variant II: TLS data goes below the TCB.
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    let tls_data_bottom = map_size;

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        map_size += round_up(startup_tls_mem_size, tls_data_align);
    }

    let header = map_size;

    map_size += size_of::<Metadata>();

    // Variant I: TLS data goes above the TCB.
    #[cfg(any(target_arch = "aarch64", target_arch = "arm", target_arch = "riscv64"))]
    {
        map_size = round_up(map_size, tls_data_align);
    }

    #[cfg(any(target_arch = "aarch64", target_arch = "arm", target_arch = "riscv64"))]
    let tls_data_bottom = map_size;

    #[cfg(any(target_arch = "aarch64", target_arch = "arm", target_arch = "riscv64"))]
    {
        map_size += round_up(startup_tls_mem_size, tls_data_align);
    }

    // Now we'll `mmap` the memory, initialize it, and create the OS thread.
    unsafe {
        // Allocate address space for the thread, including guard pages.
        let map = mmap_anonymous(
            null_mut(),
            map_size,
            ProtFlags::empty(),
            MapFlags::PRIVATE | MapFlags::STACK,
        )?
        .cast::<u8>();

        // Make the thread metadata and stack readable and writeable, leaving
        // the guard region inaccessible.
        mprotect(
            map.add(stack_bottom).cast(),
            map_size - stack_bottom,
            MprotectFlags::READ | MprotectFlags::WRITE,
        )?;

        // Compute specific pointers into the thread's memory.
        let stack = map.add(stack_top);
        let stack_least = map.add(stack_bottom);

        let tls_data = map.add(tls_data_bottom);
        let metadata: *mut Metadata = map.add(header).cast();
        let newtls: *mut Abi = &mut (*metadata).abi;

        // Initialize the thread metadata.
        ptr::write(
            metadata,
            Metadata {
                abi: Abi {
                    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
                    this: newtls,
                    #[cfg(any(target_arch = "aarch64", target_arch = "arm"))]
                    dtv: null(),
                    #[cfg(any(target_arch = "aarch64", target_arch = "arm"))]
                    pad: [0_usize; 1],
                },
                thread: ThreadData::new(
                    None, // the real tid will be written by `clone`.
                    stack_least.cast(),
                    stack_size,
                    guard_size,
                    map_size,
                ),
            },
        );

        // Initialize the TLS data with explicit initializer data.
        slice::from_raw_parts_mut(tls_data, STARTUP_TLS_INFO.file_size).copy_from_slice(
            slice::from_raw_parts(
                STARTUP_TLS_INFO.addr.cast::<u8>(),
                STARTUP_TLS_INFO.file_size,
            ),
        );

        // The TLS region includes additional data beyond `file_size` which is
        // expected to be zero-initialized, but we don't need to do anything
        // here since we allocated the memory with `mmap_anonymous` so it's
        // already zeroed.

        // Create the OS thread. In Linux, this is a process that shares much
        // of its state with the current process. We also pass additional
        // flags:
        //  - `SETTLS` to set the platform thread register.
        //  - `CHILD_CLEARTID` to arrange for a futex wait for threads waiting in
        //    `join_thread`.
        //  - `PARENT_SETTID` to store the child's tid at the `parent_tid` location.
        //  - `CHILD_SETTID` to store the child's tid at the `child_tid` location.
        // We receive the tid in the same memory for the parent and the child,
        // but we set both `PARENT_SETTID` and `CHILD_SETTID` to ensure that
        // the store completes before either the parent or child reads the tid.
        let flags = CloneFlags::VM
            | CloneFlags::FS
            | CloneFlags::FILES
            | CloneFlags::SIGHAND
            | CloneFlags::THREAD
            | CloneFlags::SYSVSEM
            | CloneFlags::SETTLS
            | CloneFlags::CHILD_CLEARTID
            | CloneFlags::CHILD_SETTID
            | CloneFlags::PARENT_SETTID;
        let thread_id_ptr = (*metadata).thread.thread_id.as_ptr();
        #[cfg(target_arch = "x86_64")]
        let clone_res = clone(
            flags.bits(),
            stack.cast(),
            thread_id_ptr,
            thread_id_ptr,
            newtls.cast::<u8>().cast(),
            Box::into_raw(Box::new(fn_)),
        );
        #[cfg(any(
            target_arch = "x86",
            target_arch = "aarch64",
            target_arch = "arm",
            target_arch = "riscv64"
        ))]
        let clone_res = clone(
            flags.bits(),
            stack.cast(),
            thread_id_ptr,
            newtls.cast::<u8>().cast(),
            thread_id_ptr,
            Box::into_raw(Box::new(fn_)),
        );
        if clone_res >= 0 {
            #[cfg(feature = "log")]
            log::trace!(
                "Thread[{:?}] launched thread Thread[{:?}] with stack_size={} and guard_size={}",
                current_thread_id(),
                clone_res,
                stack_size,
                guard_size
            );

            Ok(Thread(&mut (*metadata).thread))
        } else {
            Err(io::Errno::from_raw_os_error(-clone_res as i32))
        }
    }
}

fn round_up(addr: usize, boundary: usize) -> usize {
    (addr + (boundary - 1)) & boundary.wrapping_neg()
}

/// Marks a thread as "detached".
///
/// Detached threads free their own resources automatically when they
/// exit, rather than when they are joined.
///
/// # Safety
///
/// `thread` must point to a valid and live thread record that has not yet been
/// detached and will not be joined.
#[inline]
pub unsafe fn detach_thread(thread: Thread) {
    #[cfg(feature = "log")]
    let thread_id = (*thread.0).thread_id.load(SeqCst);

    #[cfg(feature = "log")]
    if log::log_enabled!(log::Level::Trace) {
        log::trace!(
            "Thread[{:?}] marked as detached by Thread[{:?}]",
            thread_id,
            current_thread_id()
        );
    }

    if (*thread.0).detached.swap(DETACHED, SeqCst) == ABANDONED {
        wait_for_thread_exit(thread);

        #[cfg(feature = "log")]
        log_thread_to_be_freed(thread_id);

        free_thread_memory(thread);
    }
}

/// Waits for a thread to finish.
///
/// # Safety
///
/// `thread` must point to a valid and live thread record that has not already
/// been detached or joined.
pub unsafe fn join_thread(thread: Thread) {
    #[cfg(feature = "log")]
    let thread_id = (*thread.0).thread_id.load(SeqCst);

    #[cfg(feature = "log")]
    if log::log_enabled!(log::Level::Trace) {
        log::trace!(
            "Thread[{:?}] is being joined by Thread[{:?}]",
            thread_id,
            current_thread_id()
        );
    }

    wait_for_thread_exit(thread);
    debug_assert_eq!((*thread.0).detached.load(SeqCst), ABANDONED);

    #[cfg(feature = "log")]
    log_thread_to_be_freed(thread_id);

    free_thread_memory(thread);
}

unsafe fn wait_for_thread_exit(thread: Thread) {
    use rustix::thread::{futex, FutexFlags, FutexOperation};

    // Check whether the thread has exited already; we set the
    // `CloneFlags::CHILD_CLEARTID` flag on the clone syscall, so we can test
    // for `NONE` here.
    let thread = &mut *thread.0;
    let thread_id = &mut thread.thread_id;
    let id_value = thread_id.load(SeqCst);
    if let Some(id_value) = Pid::from_raw(id_value) {
        // This doesn't use any shared memory, but we can't use
        // `FutexFlags::PRIVATE` because the wake comes from Linux
        // as arranged by the `CloneFlags::CHILD_CLEARTID` flag,
        // and Linux doesn't use the private flag for the wake.
        match futex(
            thread_id.as_ptr().cast::<u32>(),
            FutexOperation::Wait,
            FutexFlags::empty(),
            id_value.as_raw_nonzero().get() as u32,
            null(),
            null_mut(),
            0,
        ) {
            Ok(_) => {}
            Err(e) => debug_assert_eq!(e, io::Errno::AGAIN),
        }
    }
}

#[cfg(feature = "log")]
unsafe fn log_thread_to_be_freed(thread_id: i32) {
    if log::log_enabled!(log::Level::Trace) {
        log::trace!("Thread[{:?}] memory being freed", thread_id);
    }
}

unsafe fn free_thread_memory(thread: Thread) {
    use rustix::mm::munmap;

    // The thread was detached. Prepare to free the memory. First read out
    // all the fields that we'll need before freeing it.
    let map_size = (*thread.0).map_size;
    let stack_addr = (*thread.0).stack_addr;
    let guard_size = (*thread.0).guard_size;

    // Deallocate the `ThreadData`.
    drop_in_place(thread.0);

    // Free the thread's `mmap` region, if we allocated it.
    if map_size != 0 {
        let map = stack_addr.cast::<u8>().sub(guard_size);
        munmap(map.cast(), map_size).unwrap();
    }
}

/// Return the default stack size for new threads.
#[inline]
pub fn default_stack_size() -> usize {
    // This is just something simple that works for now.
    unsafe { max(page_size() * 2, STARTUP_TLS_INFO.stack_size) }
}

/// Return the default guard size for new threads.
#[inline]
pub fn default_guard_size() -> usize {
    // This is just something simple that works for now.
    page_size() * 4
}

/// Information obtained from the `DT_TLS` segment of the executable.
static mut STARTUP_TLS_INFO: StartupTlsInfo = StartupTlsInfo {
    addr: null(),
    mem_size: 0,
    file_size: 0,
    align: 0,
    stack_size: 0,
};

/// The ARM ABI expects this to be defined.
#[cfg(target_arch = "arm")]
#[no_mangle]
unsafe extern "C" fn __aeabi_read_tp() -> *mut c_void {
    get_thread_pointer()
}

// We define `clone` and `CloneFlags` here in `origin` instead of `rustix`
// because `clone` needs custom assembly code that knows about what we're
// using it for.
bitflags::bitflags! {
    struct CloneFlags: u32 {
        const NEWTIME        = linux_raw_sys::general::CLONE_NEWTIME; // since Linux 5.6
        const VM             = linux_raw_sys::general::CLONE_VM;
        const FS             = linux_raw_sys::general::CLONE_FS;
        const FILES          = linux_raw_sys::general::CLONE_FILES;
        const SIGHAND        = linux_raw_sys::general::CLONE_SIGHAND;
        const PIDFD          = linux_raw_sys::general::CLONE_PIDFD; // since Linux 5.2
        const PTRACE         = linux_raw_sys::general::CLONE_PTRACE;
        const VFORK          = linux_raw_sys::general::CLONE_VFORK;
        const PARENT         = linux_raw_sys::general::CLONE_PARENT;
        const THREAD         = linux_raw_sys::general::CLONE_THREAD;
        const NEWNS          = linux_raw_sys::general::CLONE_NEWNS;
        const SYSVSEM        = linux_raw_sys::general::CLONE_SYSVSEM;
        const SETTLS         = linux_raw_sys::general::CLONE_SETTLS;
        const PARENT_SETTID  = linux_raw_sys::general::CLONE_PARENT_SETTID;
        const CHILD_CLEARTID = linux_raw_sys::general::CLONE_CHILD_CLEARTID;
        const DETACHED       = linux_raw_sys::general::CLONE_DETACHED;
        const UNTRACED       = linux_raw_sys::general::CLONE_UNTRACED;
        const CHILD_SETTID   = linux_raw_sys::general::CLONE_CHILD_SETTID;
        const NEWCGROUP      = linux_raw_sys::general::CLONE_NEWCGROUP; // since Linux 4.6
        const NEWUTS         = linux_raw_sys::general::CLONE_NEWUTS;
        const NEWIPC         = linux_raw_sys::general::CLONE_NEWIPC;
        const NEWUSER        = linux_raw_sys::general::CLONE_NEWUSER;
        const NEWPID         = linux_raw_sys::general::CLONE_NEWPID;
        const NEWNET         = linux_raw_sys::general::CLONE_NEWNET;
        const IO             = linux_raw_sys::general::CLONE_IO;
    }
}
