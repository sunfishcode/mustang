//! Threads runtime.

use crate::arch::{clone, get_thread_pointer, munmap_current, set_thread_pointer};
use memoffset::offset_of;
use rsix::io;
use rsix::process::{getrlimit, linux_execfn, page_size, Pid, Resource};
use rsix::thread::gettid;
use rsix::thread::tls::{set_tid_address, StartupTlsInfo};
use std::cmp::max;
use std::ffi::c_void;
use std::mem::{align_of, size_of};
use std::ptr::{self, drop_in_place, null, null_mut};
use std::slice;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::atomic::{AtomicU32, AtomicU8};

/// The entrypoint where Rust code is first executed on a new thread.
///
/// This calls `func`, passing it `arg`, on the new thread. When `func`
/// returns, the thread exits.
///
/// # Safety
///
/// After calling `func`, this terminates the thread.
#[cfg(feature = "threads")]
pub(super) unsafe extern "C" fn entry(
    arg: *mut c_void,
    func: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
) -> ! {
    log::trace!(
        "Thread[{:?}] launched; calling `{:?}({:?})`",
        current_thread_id(),
        func,
        arg
    );

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
        debug_assert_eq!(builtin_return_address(0), std::ptr::null());
        debug_assert_ne!(builtin_frame_address(0), std::ptr::null());
        #[cfg(not(target_arch = "x86"))]
        debug_assert_eq!(builtin_frame_address(0) as usize & 0xf, 0);
        #[cfg(target_arch = "x86")]
        debug_assert_eq!(builtin_frame_address(0) as usize & 0xf, 8);
        debug_assert_eq!(builtin_frame_address(1), std::ptr::null());
        #[cfg(target_arch = "aarch64")]
        debug_assert_ne!(builtin_sponentry(), std::ptr::null());
        #[cfg(target_arch = "aarch64")]
        debug_assert_eq!(builtin_sponentry() as usize & 0xf, 0);

        // Check that `clone` stored our thread id as we expected.
        debug_assert_eq!(current_thread_id(), gettid());
    }

    // Call the user thread function. In `std`, this is `thread_start`. Ignore
    // the return value for now, as `std` doesn't need it.
    let _result = func(arg);

    exit_thread()
}

/// Metadata describing a thread.
#[repr(C)]
struct Metadata {
    /// Crate-internal fields. On platforms where TLS data goes after the
    /// ABI-exposed fields, we store our fields before them.
    #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
    thread: Thread,

    /// ABI-exposed fields. This is what the platform thread-pointer register
    /// points to.
    abi: Abi,

    /// Crate-internal fields. On platforms where TLS data goes before the
    /// ABI-exposed fields, we store our fields after them.
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    thread: Thread,
}

/// Fields which accessed by user code via known offsets from the platform
/// thread-pointer register.
#[repr(C)]
struct Abi {
    /// Aarch64 has an ABI-exposed `dtv` field (though we don't yet implement
    /// dynamic linking).
    #[cfg(target_arch = "aarch64")]
    dtv: *const c_void,

    /// x86 and x86-64 put a copy of the thread-pointer register at the memory
    /// location pointed to by the thread-pointer register, because reading the
    /// thread-pointer register directly is slow.
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    this: *mut Abi,

    /// Padding to put the TLS data which follows at its known offset.
    #[cfg(target_arch = "aarch64")]
    pad: [usize; 1],
}

/// Data associated with a thread. This is not `repr(C)` and not ABI-exposed.
pub struct Thread {
    thread_id: AtomicU32,
    detached: AtomicU8,
    stack_addr: *mut c_void,
    stack_size: usize,
    guard_size: usize,
    map_size: usize,
    dtors: Vec<(unsafe extern "C" fn(*mut c_void), *mut c_void)>,
}

// Values for `Thread::detached`.
const INITIAL: u8 = 0;
const DETACHED: u8 = 1;
const ABANDONED: u8 = 2;

impl Thread {
    #[inline]
    fn new(
        tid: Pid,
        stack_addr: *mut c_void,
        stack_size: usize,
        guard_size: usize,
        map_size: usize,
    ) -> Self {
        Self {
            thread_id: AtomicU32::new(tid.as_raw()),
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
fn current_abi() -> *mut Abi {
    get_thread_pointer().cast::<Abi>()
}

#[inline]
fn current_metadata() -> *mut Metadata {
    current_abi()
        .cast::<u8>()
        .wrapping_sub(offset_of!(Metadata, abi))
        .cast()
}

/// Return a raw pointer to the data associated with the current thread.
#[inline]
pub fn current_thread() -> *mut Thread {
    unsafe { &mut (*current_metadata()).thread }
}

/// Return the current thread id.
///
/// This is the same as `rsix::thread::gettid()`, but loads the value from a
/// field in the runtime rather than making a system call.
#[inline]
pub fn current_thread_id() -> Pid {
    let tid = unsafe { Pid::from_raw((*current_thread()).thread_id.load(SeqCst)) };
    debug_assert_eq!(tid, gettid(), "`current_thread_id` disagrees with `gettid`");
    tid
}

/// Return the current thread's stack address (lowest address), size, and guard
/// size.
///
/// # Safety
///
/// `thread` must point to a valid and live thread record.
#[inline]
pub unsafe fn thread_stack(thread: *mut Thread) -> (*mut c_void, usize, usize) {
    let data = &*thread;
    (data.stack_addr, data.stack_size, data.guard_size)
}

/// Registers a function to call when the current thread exits.
pub fn at_thread_exit(func: unsafe extern "C" fn(*mut c_void), obj: *mut c_void) {
    let current = current_thread();
    unsafe {
        (*current).dtors.push((func, obj));
    }
}

/// Call the destructors registered with `at_thread_exit`.
unsafe fn call_thread_dtors(current: *mut Thread) {
    // Run the `dtors`, in reverse order of registration. Note that destructors
    // may register new destructors.
    while let Some((func, obj)) = (*current).dtors.pop() {
        log::trace!(
            "Thread[{:?}] calling `at_thread_exit`-registered function `{:?}({:?})`",
            (*current).thread_id,
            func,
            obj
        );
        func(obj);
    }
}

/// Call the destructors registered with `at_thread_exit` and exit the thread.
unsafe fn exit_thread() -> ! {
    let current = current_thread();

    // Call functions registered with `at_thread_exit`.
    call_thread_dtors(current);

    // Read all the fields from the `Thread` before we release its memory.
    let current = &mut *current_thread();
    let current_thread_id = current.thread_id.load(SeqCst);
    let current_map_size = current.map_size;
    let current_stack_addr = current.stack_addr;
    let current_guard_size = current.guard_size;
    let state = current
        .detached
        .compare_exchange(INITIAL, ABANDONED, SeqCst, SeqCst);

    // Deallocate the `Thread`.
    drop_in_place(current);
    drop(current);

    // If we're still in the `INITIAL` state, switch to `ABANDONED`, which
    // tells `join_thread` to free the memory. Otherwise, we're in the
    // `DETACHED` state, and we free the memory immediately.
    if let Err(e) = state {
        log::trace!("Thread[{:?}] exiting as detached", current_thread_id);
        debug_assert_eq!(e, DETACHED);

        // Free the thread's `mmap` region, if we allocated it.
        let map_size = current_map_size;
        if map_size != 0 {
            // NULL out the tid address, so the kernel doesn't write to memory
            // that we've freed trying to clear our TID when we exit.
            let _ = set_tid_address(null_mut());
            // `munmap` the memory, which also frees the stack we're currently
            // on, and do an `exit` carefully without touching the stack.
            let map = current_stack_addr.cast::<u8>().sub(current_guard_size);
            munmap_and_exit_thread(map.cast(), map_size);
        }
    } else {
        log::trace!("Thread[{:?}] exiting as joinable", current_thread_id);
    }

    // Terminate the thread.
    rsix::thread::tls::exit_thread(0)
}

/// Initialize the main thread.
///
/// This function is similar to `create_thread` except that the OS thread is
/// already created, and already has a stack (which we need to locate), and is
/// already running.
pub(super) unsafe fn initialize_main_thread(mem: *mut c_void) {
    // Read the TLS information from the ELF header.
    STARTUP_TLS_INFO = rsix::thread::tls::startup_tls_info();

    // Determine the top of the stack. Linux puts the `AT_EXECFN` string at
    // the top, so find the end of that, and then round up to the page size.
    // See <https://lwn.net/Articles/631631/> for details.
    let execfn = linux_execfn().to_bytes_with_nul();
    let stack_base = execfn.as_ptr().add(execfn.len());
    let stack_base = round_up(stack_base as usize, page_size()) as *mut c_void;

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
    #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
    {
        alloc_size = round_up(alloc_size, tls_data_align);
    }

    #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
    let tls_data_bottom = alloc_size;

    #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
    {
        alloc_size += round_up(STARTUP_TLS_INFO.mem_size, tls_data_align);
    }

    let layout = std::alloc::Layout::from_size_align(alloc_size, metadata_align).unwrap();

    // Allocate the thread data.
    let new = std::alloc::alloc(layout);

    let tls_data = new.add(tls_data_bottom);
    let metadata: *mut Metadata = new.add(header).cast();
    let newtls = (&mut (*metadata).abi) as *mut Abi;

    // Initialize the thread metadata.
    ptr::write(
        metadata,
        Metadata {
            abi: Abi {
                #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
                this: newtls,
                #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
                dtv: null(),
                #[cfg(target_arch = "aarch64")]
                pad: [0_usize; 1],
            },
            thread: Thread::new(
                gettid(),
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
    set_thread_pointer(newtls.cast());
    assert_eq!(newtls, current_abi().cast());
}

/// Creates a new thread.
///
/// `fn_` is called on the new thread and passed `arg`.
///
/// This is a safe function because it itself doesn't dereference `arg`; it
/// calls `fn_`, which does, and it's marked `unsafe`.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn create_thread(
    fn_: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
    arg: *mut c_void,
    stack_size: usize,
    guard_size: usize,
) -> io::Result<*mut Thread> {
    use io::{mmap_anonymous, mprotect, MapFlags, MprotectFlags, ProtFlags};

    // Safety: `STARGUP_TLS_INFO` is initialized at program startup before
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
    #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
    {
        map_size = round_up(map_size, tls_data_align);
    }

    #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
    let tls_data_bottom = map_size;

    #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
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
        let newtls = (&mut (*metadata).abi) as *mut Abi;

        // Initialize the thread metadata.
        ptr::write(
            metadata,
            Metadata {
                abi: Abi {
                    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
                    this: newtls,
                    #[cfg(target_arch = "aarch64")]
                    dtv: null(),
                    #[cfg(target_arch = "aarch64")]
                    pad: [0_usize; 1],
                },
                thread: Thread::new(
                    Pid::NONE, // the real tid will be written by `clone`.
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
        let thread_id_ptr = (*metadata).thread.thread_id.as_mut_ptr();
        #[cfg(target_arch = "x86_64")]
        let clone_res = clone(
            flags.bits(),
            stack.cast(),
            thread_id_ptr,
            thread_id_ptr,
            newtls.cast(),
            arg,
            fn_,
        );
        #[cfg(any(target_arch = "x86", target_arch = "aarch64", target_arch = "riscv64"))]
        let clone_res = clone(
            flags.bits(),
            stack.cast(),
            thread_id_ptr,
            newtls.cast(),
            thread_id_ptr,
            arg,
            fn_,
        );
        if clone_res >= 0 {
            Ok(&mut (*metadata).thread)
        } else {
            Err(io::Error::from_raw_os_error(-clone_res as i32))
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
pub unsafe fn detach_thread(thread: *mut Thread) {
    if (*thread).detached.swap(DETACHED, SeqCst) == ABANDONED {
        wait_for_thread_exit(thread);
        free_thread_memory(thread);
    }
}

/// Waits for a thread to finish.
///
/// # Safety
///
/// `thread` must point to a valid and live thread record that has not already
/// been detached or joined.
pub unsafe fn join_thread(thread: *mut Thread) {
    wait_for_thread_exit(thread);
    debug_assert_eq!((*thread).detached.load(SeqCst), ABANDONED);
    free_thread_memory(thread);
}

unsafe fn wait_for_thread_exit(thread: *mut Thread) {
    use rsix::thread::{futex, FutexFlags, FutexOperation};

    // Check whether the thread has exited already; we set the
    // `CloneFlags::CHILD_CLEARTID` flag on the clone syscall, so we can test
    // for `NONE` here.
    let thread = &mut *thread;
    let thread_id = &mut thread.thread_id;
    let id_value = thread_id.load(SeqCst);
    if Pid::from_raw(id_value) != Pid::NONE {
        // This doesn't use any shared memory, but we can't use
        // `FutexFlags::PRIVATE` because the wake comes from Linux
        // as arranged by the `CloneFlags::CHILD_CLEARTID` flag,
        // and Linux doesn't use the private flag for the wake.
        match futex(
            thread_id.as_mut_ptr(),
            FutexOperation::Wait,
            FutexFlags::empty(),
            id_value,
            null(),
            null_mut(),
            0,
        ) {
            Ok(_) => {}
            Err(e) => debug_assert_eq!(e, io::Error::AGAIN),
        }
    }
}

unsafe fn free_thread_memory(thread: *mut Thread) {
    use rsix::io::munmap;

    // Free the thread's `mmap` region, if we allocated it.
    let thread = &mut *thread;
    let map_size = thread.map_size;
    if map_size != 0 {
        let map = thread.stack_addr.cast::<u8>().sub(thread.guard_size);
        drop(thread);
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

// We define `clone` and `CloneFlags` here in `origin` instead of `rsix`
// because `clone` needs custom assembly code that knows about what we're
// using it for.
bitflags::bitflags! {
    struct CloneFlags: u32 {
        const NEWTIME        = linux_raw_sys::v5_11::general::CLONE_NEWTIME; // since Linux 5.6
        const VM             = linux_raw_sys::general::CLONE_VM;
        const FS             = linux_raw_sys::general::CLONE_FS;
        const FILES          = linux_raw_sys::general::CLONE_FILES;
        const SIGHAND        = linux_raw_sys::general::CLONE_SIGHAND;
        const PIDFD          = linux_raw_sys::v5_4::general::CLONE_PIDFD; // since Linux 5.2
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
        const NEWCGROUP      = linux_raw_sys::v5_4::general::CLONE_NEWCGROUP; // since Linux 4.6
        const NEWUTS         = linux_raw_sys::general::CLONE_NEWUTS;
        const NEWIPC         = linux_raw_sys::general::CLONE_NEWIPC;
        const NEWUSER        = linux_raw_sys::general::CLONE_NEWUSER;
        const NEWPID         = linux_raw_sys::general::CLONE_NEWPID;
        const NEWNET         = linux_raw_sys::general::CLONE_NEWNET;
        const IO             = linux_raw_sys::general::CLONE_IO;
    }
}
