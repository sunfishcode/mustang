use parking_lot::lock_api::RawMutex as _;
use parking_lot::{Mutex, RawMutex};

/// Functions registered with `at_fork`.
static FORK_FUNCS: Mutex<RegisteredForkFuncs> =
    Mutex::const_new(RawMutex::INIT, RegisteredForkFuncs::new());

/// A type for holding `fork` callbacks.
#[derive(Default)]
struct RegisteredForkFuncs {
    /// Functions called before calling `fork`.
    pub(crate) prepare: Vec<unsafe extern "C" fn()>,

    /// Functions called after calling `fork`, in the parent.
    pub(crate) parent: Vec<unsafe extern "C" fn()>,

    /// Functions called after calling `fork`, in the child.
    pub(crate) child: Vec<unsafe extern "C" fn()>,
}

impl RegisteredForkFuncs {
    pub(crate) const fn new() -> Self {
        Self {
            prepare: Vec::new(),
            parent: Vec::new(),
            child: Vec::new(),
        }
    }
}

/// Register functions to be called when `fork` is called.
///
/// The handlers for each phase are called in the following order:
/// the prepare handlers are called in reverse order of registration;
/// the parent and child handlers are called in the order of registration.
pub(crate) fn at_fork(
    prepare_func: Option<unsafe extern "C" fn()>,
    parent_func: Option<unsafe extern "C" fn()>,
    child_func: Option<unsafe extern "C" fn()>,
) {
    let mut funcs = FORK_FUNCS.lock();

    // Add the callbacks to the lists.
    funcs.prepare.extend(prepare_func);
    funcs.parent.extend(parent_func);
    funcs.child.extend(child_func);
}

pub(crate) unsafe fn fork() -> rustix::io::Result<Option<rustix::process::Pid>> {
    let funcs = FORK_FUNCS.lock();

    // Callbacks before calling `fork`.
    funcs.prepare.iter().rev().for_each(|func| func());

    // Call `fork`.
    match rustix::runtime::fork()? {
        None => {
            // The child's thread record is copied from the parent;
            // update it with the child's current-thread-id.
            #[cfg(feature = "threads")]
            origin::set_current_thread_id_after_a_fork(rustix::thread::gettid());

            // Callbacks after calling `fork`, in the child.
            funcs.child.iter().for_each(|func| func());
            Ok(None)
        }
        Some(pid) => {
            // Callbacks after calling `fork`, in the parent.
            funcs.parent.iter().for_each(|func| func());
            Ok(Some(pid))
        }
    }
}
