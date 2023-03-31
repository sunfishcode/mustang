use crate::{convert_res, set_errno, Errno};
use libc::{c_int, c_uint, RLIM_INFINITY};
use rustix::process::{Pid, Resource, Rlimit};

// TODO: Upstream this to libc.
#[cfg(target_os = "linux")]
const RLIM64_INFINITY: libc::rlim64_t = !0;

#[no_mangle]
unsafe extern "C" fn prlimit(
    pid: libc::pid_t,
    resource: c_uint,
    new_limit: *const libc::rlimit,
    old_limit: *mut libc::rlimit,
) -> c_int {
    libc!(libc::prlimit(pid, resource, new_limit, old_limit));

    let resource = match resource_to_rustix(resource) {
        Some(resource) => resource,
        None => return -1,
    };
    let new_limit = rlimit_to_rustix(*new_limit);
    let pid = Pid::from_raw(pid as _);
    match convert_res(rustix::process::prlimit(pid, resource, new_limit)) {
        Some(limit) => {
            if !old_limit.is_null() {
                *old_limit = rustix_to_rlimit(limit);
            }
            0
        }
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn prlimit64(
    pid: libc::pid_t,
    resource: c_uint,
    new_limit: *const libc::rlimit64,
    old_limit: *mut libc::rlimit64,
) -> c_int {
    libc!(libc::prlimit64(pid, resource, new_limit, old_limit));

    let resource = match resource_to_rustix(resource) {
        Some(resource) => resource,
        None => return -1,
    };
    let new_limit = rlimit64_to_rustix(*new_limit);
    let pid = Pid::from_raw(pid as _);
    match convert_res(rustix::process::prlimit(pid, resource, new_limit)) {
        Some(limit) => {
            if !old_limit.is_null() {
                *old_limit = rustix_to_rlimit64(limit);
            }
            0
        }
        None => -1,
    }
}

fn resource_to_rustix(resource: libc::c_uint) -> Option<Resource> {
    Some(match resource {
        libc::RLIMIT_CPU => Resource::Cpu,
        libc::RLIMIT_FSIZE => Resource::Fsize,
        libc::RLIMIT_DATA => Resource::Data,
        libc::RLIMIT_STACK => Resource::Stack,
        libc::RLIMIT_CORE => Resource::Core,
        libc::RLIMIT_RSS => Resource::Rss,
        libc::RLIMIT_NPROC => Resource::Nproc,
        libc::RLIMIT_NOFILE => Resource::Nofile,
        libc::RLIMIT_MEMLOCK => Resource::Memlock,
        libc::RLIMIT_AS => Resource::As,
        libc::RLIMIT_LOCKS => Resource::Locks,
        libc::RLIMIT_SIGPENDING => Resource::Sigpending,
        libc::RLIMIT_MSGQUEUE => Resource::Msgqueue,
        libc::RLIMIT_NICE => Resource::Nice,
        libc::RLIMIT_RTPRIO => Resource::Rtprio,
        libc::RLIMIT_RTTIME => Resource::Rttime,
        _ => {
            set_errno(Errno(libc::EINVAL));
            return None;
        }
    })
}

fn rlimit_to_rustix(limit: libc::rlimit) -> Rlimit {
    Rlimit {
        current: if limit.rlim_cur == RLIM_INFINITY {
            None
        } else {
            Some(limit.rlim_cur)
        },
        maximum: if limit.rlim_max == RLIM_INFINITY {
            None
        } else {
            Some(limit.rlim_max)
        },
    }
}

fn rlimit64_to_rustix(limit: libc::rlimit64) -> Rlimit {
    Rlimit {
        current: if limit.rlim_cur == RLIM64_INFINITY {
            None
        } else {
            Some(limit.rlim_cur)
        },
        maximum: if limit.rlim_max == RLIM64_INFINITY {
            None
        } else {
            Some(limit.rlim_max)
        },
    }
}

fn rustix_to_rlimit(limit: Rlimit) -> libc::rlimit {
    libc::rlimit {
        rlim_cur: match limit.current {
            Some(lim) => lim,
            None => RLIM_INFINITY,
        },
        rlim_max: match limit.maximum {
            Some(lim) => lim,
            None => RLIM_INFINITY,
        },
    }
}

fn rustix_to_rlimit64(limit: Rlimit) -> libc::rlimit64 {
    libc::rlimit64 {
        rlim_cur: match limit.current {
            Some(lim) => lim,
            None => RLIM64_INFINITY,
        },
        rlim_max: match limit.maximum {
            Some(lim) => lim,
            None => RLIM64_INFINITY,
        },
    }
}
