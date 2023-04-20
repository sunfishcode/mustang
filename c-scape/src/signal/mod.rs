use crate::convert_res;
use core::mem::{size_of, size_of_val, transmute, zeroed};
use core::ptr::copy_nonoverlapping;
use errno::{set_errno, Errno};
use libc::*;

#[no_mangle]
unsafe extern "C" fn signal(signal: c_int, handler: sighandler_t) -> sighandler_t {
    libc!(libc::signal(signal, handler));

    use rustix::process::Signal;
    use rustix::runtime::Sigaction;

    let signal = Signal::from_raw(signal).unwrap();
    let mut new = zeroed::<Sigaction>();
    new.sa_handler_kernel = transmute(handler);
    new.sa_flags = SA_RESTART as _;

    match rustix::runtime::sigaction(signal, Some(new)) {
        Ok(old) => transmute(old.sa_handler_kernel),
        Err(_) => SIG_ERR,
    }
}

#[no_mangle]
unsafe extern "C" fn sigaction(signal: c_int, new: *const sigaction, old: *mut sigaction) -> c_int {
    libc!(libc::sigaction(signal, new, old));

    use rustix::process::Signal;
    use rustix::runtime::Sigaction;

    let signal = Signal::from_raw(signal).unwrap();
    let new = if new.is_null() {
        None
    } else {
        let new = new.read();
        let mut sa_mask = zeroed();
        copy_nonoverlapping(
            &new.sa_mask as *const sigset_t as *const u8,
            &mut sa_mask as *mut _ as *mut u8,
            size_of_val(&zeroed::<Sigaction>().sa_mask),
        );

        let new = Sigaction {
            sa_handler_kernel: transmute(new.sa_sigaction),
            sa_flags: new.sa_flags.try_into().unwrap(),
            sa_restorer: transmute(new.sa_sigaction),
            sa_mask,
        };

        Some(new)
    };

    match convert_res(rustix::runtime::sigaction(signal, new)) {
        Some(old_action) => {
            if !old.is_null() {
                let mut sa_mask = zeroed();
                copy_nonoverlapping(
                    &old_action.sa_mask as *const _ as *const u8,
                    &mut sa_mask as *mut sigset_t as *mut u8,
                    size_of_val(&zeroed::<Sigaction>().sa_mask),
                );

                let old_action = sigaction {
                    sa_sigaction: transmute(old_action.sa_handler_kernel),
                    sa_flags: old_action.sa_flags.try_into().unwrap(),
                    sa_restorer: transmute(old_action.sa_restorer),
                    sa_mask,
                };
                old.write(old_action);
            }
            0
        }
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn sigaltstack(new: *const stack_t, old: *mut stack_t) -> c_int {
    libc!(libc::sigaltstack(new, old));

    let new: *const rustix::runtime::Stack = checked_cast!(new);
    let old: *mut rustix::runtime::Stack = checked_cast!(old);

    let new = if new.is_null() {
        None
    } else {
        Some(new.read())
    };

    match convert_res(rustix::runtime::sigaltstack(new)) {
        Some(old_stack) => {
            if !old.is_null() {
                old.write(old_stack);
            }
            0
        }
        None => -1,
    }
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn raise(sig: c_int) -> c_int {
    libc!(libc::raise(sig));

    let sig = rustix::process::Signal::from_raw(sig).unwrap();
    match convert_res(rustix::runtime::tkill(rustix::thread::gettid(), sig)) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn abort() {
    libc!(libc::abort());

    rustix::runtime::tkill(rustix::thread::gettid(), rustix::process::Signal::Abort).ok();

    unimplemented!("`abort()` when `SIGABRT` is ignored")
}

#[no_mangle]
unsafe extern "C" fn sigaddset(sigset: *mut sigset_t, signum: c_int) -> c_int {
    libc!(libc::sigaddset(sigset, signum));

    if signum == 0 || signum as usize - 1 >= size_of::<sigset_t>() * 8 {
        set_errno(Errno(libc::EINVAL));
        return -1;
    }

    let sig_index = (signum - 1) as usize;
    let mut x = sigset.cast::<usize>().add(sig_index / 8).read();
    x |= 1_usize << (sig_index % (8 * size_of::<usize>()));
    sigset.cast::<usize>().add(sig_index / 8).write(x);
    0
}

#[no_mangle]
unsafe extern "C" fn sigdelset(sigset: *mut sigset_t, signum: c_int) -> c_int {
    libc!(libc::sigdelset(sigset, signum));

    if signum == 0 || signum as usize - 1 >= size_of::<sigset_t>() * 8 {
        set_errno(Errno(libc::EINVAL));
        return -1;
    }

    let sig_index = (signum - 1) as usize;
    let mut x = sigset.cast::<usize>().add(sig_index / 8).read();
    x &= !(1_usize << (sig_index % (8 * size_of::<usize>())));
    sigset.cast::<usize>().add(sig_index / 8).write(x);
    0
}

#[no_mangle]
unsafe extern "C" fn sigemptyset(sigset: *mut sigset_t) -> c_int {
    libc!(libc::sigemptyset(sigset));
    sigset.write(zeroed());
    0
}

#[no_mangle]
unsafe extern "C" fn sigfillset(sigset: *mut sigset_t) -> c_int {
    libc!(libc::sigfillset(sigset));
    for index in 0..(size_of::<sigset_t>() / size_of::<usize>()) {
        sigset.cast::<usize>().add(index).write(!0);
    }
    0
}

#[no_mangle]
unsafe extern "C" fn sigismember(sigset: *const sigset_t, signum: c_int) -> c_int {
    libc!(libc::sigismember(sigset, signum));

    if signum == 0 || signum as usize - 1 >= size_of::<sigset_t>() * 8 {
        set_errno(Errno(libc::EINVAL));
        return -1;
    }

    let sig_index = (signum - 1) as usize;
    let x = sigset.cast::<usize>().add(sig_index / 8).read();
    ((x & (1_usize << (sig_index % (8 * size_of::<usize>())))) != 0) as c_int
}
