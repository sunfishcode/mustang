use rustix::io;
use rustix::process::Signal;
#[cfg(not(target_arch = "riscv64"))]
use {
    crate::arch,
    linux_raw_sys::ctypes::c_ulong,
    linux_raw_sys::general::{SA_RESTORER, SA_SIGINFO},
};

/// A signal action record for use with [`sigaction`].
pub type Sigaction = linux_raw_sys::general::kernel_sigaction;

/// Register a signal handler.
///
/// # Safety
///
/// yolo
pub unsafe fn sigaction(sig: Signal, action: Option<Sigaction>) -> io::Result<Sigaction> {
    #[allow(unused_mut)]
    let mut action = action;

    #[cfg(not(target_arch = "riscv64"))]
    if let Some(action) = &mut action {
        action.sa_flags |= SA_RESTORER as c_ulong;

        if (action.sa_flags & SA_SIGINFO as c_ulong) == SA_SIGINFO as c_ulong {
            action.sa_restorer = Some(arch::return_from_signal_handler);
        } else {
            action.sa_restorer = Some(arch::return_from_signal_handler_noinfo);
        }
    }

    rustix::runtime::sigaction(sig, action)
}
