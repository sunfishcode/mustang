//! Threads runtime.

use crate::arch::{clone, deallocate_thread, set_thread_pointer};
use rsix::io;
use rsix::process::{getrlimit, linux_execfn, page_size, Pid, Resource};
use rsix::thread::gettid;
use rsix::thread::tls::StartupTlsInfo;
use std::cmp::max;
use std::ffi::c_void;
use std::mem::{align_of, size_of};
use std::ptr::{self, null, null_mut};
use std::slice;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::atomic::{AtomicBool, AtomicU32};

/// The data structure pointed to by the platform thread pointer register.
#[repr(C)]
struct ThreadPointee {
    /// ABI-exposed fields.
    abi: Abi,

    /// Crate-internal fields.
    data: Thread,
}

/// This holds fields which are accessed by user code from a fixed offset from
/// the platform thread pointer register.
#[repr(C)]
struct Abi {
    /// On x86 and x86-64, architectures which use "Variant II" of the
    /// [ELF TLS ABI], and the first field is a copy of the thread pointer
    /// register, since reading the register directly is expensive.
    ///
    /// [ELF TLS ABI]: https://www.akkadia.org/drepper/tls.pdf
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    this: *mut ThreadPointee,

    /// Reserve some extra storage for future use.
    reserved: [usize; 23],
}

/// Data associated with a thread.
pub struct Thread {
    thread_id: AtomicU32,
    detached: AtomicBool,
    stack_addr: *mut c_void,
    stack_size: usize,
    guard_size: usize,
    map_size: usize,
    dtors: Vec<(unsafe extern "C" fn(*mut c_void), *mut c_void)>,
}

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
            detached: AtomicBool::new(false),
            stack_addr,
            stack_size,
            guard_size,
            map_size,
            dtors: Vec::new(),
        }
    }

    /// Return the current thread's stack address (lowest address), size, and
    /// guard size.
    ///
    /// # Safety
    ///
    /// `thread_data` must point to a valid and live thread record.
    #[inline]
    pub unsafe fn stack(thread: *mut Self) -> (*mut c_void, usize, usize) {
        let data = &*thread;
        (data.stack_addr, data.stack_size, data.guard_size)
    }
}

#[inline]
fn thread_pointee() -> *mut ThreadPointee {
    crate::arch::thread_self().cast::<ThreadPointee>()
}

/// Return a raw pointer to the data associated with the current thread.
#[inline]
pub fn current_thread() -> *mut Thread {
    unsafe { &mut (*thread_pointee()).data }
}

/// Return the current thread id.
///
/// This is the same as `rsix::thread::gettid()`, but loads the value from
/// a field in the runtime rather than making a system call.
#[inline]
pub fn current_thread_id() -> Pid {
    let tid = unsafe { Pid::from_raw((*current_thread()).thread_id.load(SeqCst)) };
    debug_assert_ne!(
        tid,
        Pid::NONE,
        "`current_thread_id` called before initialization"
    );
    tid
}

/// Registers a function to call when the current thread exits.
pub fn at_thread_exit(func: unsafe extern "C" fn(*mut c_void), obj: *mut c_void) {
    unsafe {
        (*current_thread()).dtors.push((func, obj));
    }
}

/// Call the destructors registered with `at_thread_exit`.
unsafe fn call_thread_dtors(current: &mut Thread) {
    let dtors = &mut current.dtors;

    // Run the `dtors`, in reverse order of registration.
    while let Some((func, obj)) = dtors.pop() {
        func(obj);
    }

    // Free the `dtors` memory.
    *dtors = Vec::new();
}

/// Call the destructors registered with `at_thread_exit` and exit the thread.
pub(super) unsafe fn exit_thread() -> ! {
    let current = &mut *current_thread();

    // Call functions registered with `at_thread_exit`.
    call_thread_dtors(current);

    if current.detached.load(SeqCst) {
        // Free the thread's `mmap` region, if we allocated it.
        let map_size = current.map_size;
        if map_size != 0 {
            let map = current.stack_addr.cast::<u8>().sub(current.guard_size);
            // `munmap` the memory, which also frees the stack we're currently
            // on, and do an `exit` carefully without touching the stack.
            deallocate_thread(map.cast(), map_size);
        }
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
    let stack_last = stack_base.cast::<u8>().sub(stack_map_size);
    let stack_size = stack_last.offset_from(mem.cast::<u8>()) as usize;
    let guard_size = 0;
    let map_size = 0;

    // Compute relevant alignments.
    let tls_data_align = STARTUP_TLS_INFO.align;
    let header_align = align_of::<ThreadPointee>();
    let metadata_align = max(tls_data_align, header_align);

    // Compute the size to allocate for thread data.
    let mut alloc_size = 0;

    let tls_data_bottom = alloc_size;

    alloc_size += round_up(STARTUP_TLS_INFO.mem_size, tls_data_align);

    let header = alloc_size;

    alloc_size += size_of::<ThreadPointee>();

    let layout = std::alloc::Layout::from_size_align(alloc_size, metadata_align).unwrap();

    // Allocate the thread data.
    let new = std::alloc::alloc(layout);

    let tls_data = new.add(tls_data_bottom);
    let newtls: *mut ThreadPointee = new.add(header).cast();

    // Initialize the thread pointee struct.
    ptr::write(
        newtls,
        ThreadPointee {
            abi: Abi {
                #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
                this: newtls,
                reserved: [0_usize; 23],
            },
            data: Thread::new(
                gettid(),
                stack_last.cast(),
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

    // Set the platform thread point to point to the new thread pointee.
    set_thread_pointer(newtls.cast());
    assert_eq!(newtls, thread_pointee().cast());

    eprintln!(".｡oO(Threads spun up by origin! 🧵)");
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
    let header_align = align_of::<ThreadPointee>();
    let metadata_align = max(tls_data_align, header_align);
    let stack_metadata_align = max(stack_align, metadata_align);
    assert!(stack_metadata_align <= page_align);

    // Compute the `mmap` size.
    let mut map_size = 0;

    map_size += round_up(guard_size, page_align);

    let stack_bottom = map_size;

    map_size += round_up(stack_size, stack_metadata_align);

    let stack_top = map_size;
    let tls_data_bottom = map_size;

    map_size += round_up(startup_tls_mem_size, tls_data_align);

    let header = map_size;

    map_size += size_of::<ThreadPointee>();

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
        let newtls = map.add(header).cast();

        // Initialize the thread metadata.
        ptr::write(
            newtls,
            ThreadPointee {
                abi: Abi {
                    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
                    this: newtls,
                    reserved: [0_usize; 23],
                },
                data: Thread::new(
                    Pid::NONE, // the real tid will be written by `clone`.
                    stack_least.cast(),
                    stack_size,
                    guard_size,
                    map_size,
                ),
            },
        );
        let data = &mut (*newtls.cast::<ThreadPointee>()).data;

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
        // of its state with the current process. We also pass additional flags:
        // `SETTLS` to set the platform thread register.
        // `CHILD_CLEARTID` to arrange for a futex wait for threads waiting in
        // `join_thread`.
        // `PARENT_SETTID` to store the child's tid at the `parent_tid` location.
        let flags = CloneFlags::VM
            | CloneFlags::FS
            | CloneFlags::FILES
            | CloneFlags::SIGHAND
            | CloneFlags::THREAD
            | CloneFlags::SYSVSEM
            | CloneFlags::SETTLS
            | CloneFlags::CHILD_CLEARTID
            | CloneFlags::PARENT_SETTID;
        #[cfg(target_arch = "x86_64")]
        let clone_res = clone(
            flags.bits(),
            stack.cast(),
            data.thread_id.as_mut_ptr(),
            data.thread_id.as_mut_ptr(),
            newtls.cast(),
            arg,
            fn_,
        );
        #[cfg(any(target_arch = "x86", target_arch = "aarch64"))]
        let clone_res = clone(
            flags.bits(),
            stack.cast(),
            data.thread_id.as_mut_ptr(),
            newtls.cast(),
            data.thread_id.as_mut_ptr(),
            arg,
            fn_,
        );
        if clone_res >= 0 {
            Ok(data)
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
/// `thread_data` must point to a valid and live thread record that has
/// not yet been detached and will not be joined.
#[inline]
pub unsafe fn detach_thread(thread_data: *mut Thread) {
    (*thread_data).detached.store(true, SeqCst);
}

/// Waits for a thread to finish.
///
/// # Safety
///
/// `thread_data` must point to a valid and live thread record.
pub unsafe fn join_thread(thread_data: *mut Thread) -> io::Result<()> {
    use rsix::io::munmap;
    use rsix::thread::{futex, FutexFlags, FutexOperation};

    let thread_data = &mut *thread_data;
    let thread_id = &mut thread_data.thread_id;

    // Check whether the thread has exited already; we set the
    // `CloneFlags::CHILD_CLEARTID` flag on the clone syscall, so we can test
    // for `NONE` here.
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
            Ok(_) | Err(io::Error::AGAIN) => {}
            Err(e) => return Err(e),
        }
    }

    if !thread_data.detached.load(SeqCst) {
        // Free the thread's `mmap` region, if we allocated it.
        let map_size = thread_data.map_size;
        if map_size != 0 {
            let map = thread_data
                .stack_addr
                .cast::<u8>()
                .sub(thread_data.guard_size);
            munmap(map.cast(), map_size).unwrap();
        }
    }

    Ok(())
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

// TODO: Move these into `rsix`?
bitflags::bitflags! {
    struct CloneFlags: u32 {
        const NEWTIME        = 0x0000_0080;
        const VM             = 0x0000_0100;
        const FS             = 0x0000_0200;
        const FILES          = 0x0000_0400;
        const SIGHAND        = 0x0000_0800;
        const PIDFD          = 0x0000_1000;
        const PTRACE         = 0x0000_2000;
        const VFORK          = 0x0000_4000;
        const PARENT         = 0x0000_8000;
        const THREAD         = 0x0001_0000;
        const NEWNS          = 0x0002_0000;
        const SYSVSEM        = 0x0004_0000;
        const SETTLS         = 0x0008_0000;
        const PARENT_SETTID  = 0x0010_0000;
        const CHILD_CLEARTID = 0x0020_0000;
        const DETACHED       = 0x0040_0000;
        const UNTRACED       = 0x0080_0000;
        const CHILD_SETTID   = 0x0100_0000;
        const NEWCGROUP      = 0x0200_0000;
        const NEWUTS         = 0x0400_0000;
        const NEWIPC         = 0x0800_0000;
        const NEWUSER        = 0x1000_0000;
        const NEWPID         = 0x2000_0000;
        const NEWNET         = 0x4000_0000;
        const IO             = 0x8000_0000;
    }
}
