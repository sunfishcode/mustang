use crate::convert_res;
use core::ffi::CStr;
use core::mem::{size_of, zeroed};
#[cfg(feature = "take-charge")]
use core::ptr;
use core::ptr::null_mut;
#[cfg(feature = "take-charge")]
use libc::c_ulong;
use libc::{c_char, c_int, c_long, c_void};

#[no_mangle]
unsafe extern "C" fn sysconf(name: c_int) -> c_long {
    libc!(libc::sysconf(name));

    #[cfg(feature = "std")] // These are defined in c-gull.
    #[cfg(not(target_os = "wasi"))]
    extern "C" {
        fn get_nprocs_conf() -> c_int;
        fn get_nprocs() -> c_int;
        fn get_phys_pages() -> c_long;
        fn get_avphys_pages() -> c_long;
    }

    match name {
        libc::_SC_PAGESIZE => rustix::param::page_size() as _,
        #[cfg(not(target_os = "wasi"))]
        libc::_SC_GETPW_R_SIZE_MAX => -1,
        #[cfg(any(target_os = "android", target_os = "linux", target_os = "wasi"))]
        libc::_SC_SYMLOOP_MAX => 40,
        libc::_SC_HOST_NAME_MAX => 255,
        libc::_SC_NGROUPS_MAX => 32,
        #[cfg(feature = "std")]
        #[cfg(not(target_os = "wasi"))]
        libc::_SC_NPROCESSORS_CONF => get_nprocs_conf().into(),
        #[cfg(feature = "std")]
        #[cfg(not(target_os = "wasi"))]
        libc::_SC_NPROCESSORS_ONLN => get_nprocs().into(),
        #[cfg(feature = "std")]
        #[cfg(not(target_os = "wasi"))]
        libc::_SC_PHYS_PAGES => get_phys_pages(),
        #[cfg(feature = "std")]
        #[cfg(not(target_os = "wasi"))]
        libc::_SC_AVPHYS_PAGES => get_avphys_pages(),
        _ => panic!("unrecognized sysconf({})", name),
    }
}

// `getauxval` usually returns `unsigned long`, but we make it a pointer type
// so that it preserves provenance.
//
// This is not used in coexist-with-libc configurations because libc startup
// code sometimes needs to call `getauxval` before rustix is initialized.
#[cfg(feature = "take-charge")]
#[no_mangle]
unsafe extern "C" fn getauxval(type_: c_ulong) -> *mut c_void {
    libc!(ptr::from_exposed_addr_mut(libc::getauxval(type_) as _));
    unimplemented!("unrecognized getauxval {}", type_)
}

// As with `getauxval`, this is not used in coexist-with-libc configurations
// because libc startup code sometimes needs to call `getauxval` before rustix
// is initialized.
#[cfg(target_arch = "aarch64")]
#[cfg(feature = "take-charge")]
#[no_mangle]
unsafe extern "C" fn __getauxval(type_: c_ulong) -> *mut c_void {
    //libc!(ptr::from_exposed_addr(libc::__getauxval(type_) as _));
    match type_ {
        libc::AT_HWCAP => ptr::invalid_mut(rustix::param::linux_hwcap().0),
        _ => unimplemented!("unrecognized __getauxval {}", type_),
    }
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn dl_iterate_phdr(
    callback: Option<
        unsafe extern "C" fn(
            info: *mut libc::dl_phdr_info,
            size: usize,
            data: *mut c_void,
        ) -> c_int,
    >,
    data: *mut c_void,
) -> c_int {
    extern "C" {
        static mut __executable_start: c_void;
    }

    libc!(libc::dl_iterate_phdr(callback, data));

    let (phdr, _phent, phnum) = rustix::runtime::exe_phdrs();
    let mut info = libc::dl_phdr_info {
        dlpi_addr: (&mut __executable_start as *mut c_void).expose_addr() as _,
        dlpi_name: b"/proc/self/exe\0".as_ptr().cast(),
        dlpi_phdr: phdr.cast(),
        dlpi_phnum: phnum.try_into().unwrap(),
        ..zeroed()
    };
    callback.unwrap()(&mut info, size_of::<libc::dl_phdr_info>(), data)
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void {
    libc!(libc::dlsym(handle, symbol));

    if handle.is_null() {
        // `std` uses `dlsym` to dynamically detect feature availability; recognize
        // functions it asks for.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        if CStr::from_ptr(symbol.cast()).to_bytes() == b"statx" {
            return libc::statx as *mut c_void;
        }
        #[cfg(any(target_os = "android", target_os = "linux"))]
        if CStr::from_ptr(symbol.cast()).to_bytes() == b"getrandom" {
            return libc::getrandom as *mut c_void;
        }
        #[cfg(any(target_os = "android", target_os = "linux"))]
        if CStr::from_ptr(symbol.cast()).to_bytes() == b"copy_file_range" {
            return libc::copy_file_range as *mut c_void;
        }
        #[cfg(any(target_os = "android", target_os = "linux"))]
        if CStr::from_ptr(symbol.cast()).to_bytes() == b"clone3" {
            // Let's just say we don't support this for now.
            return null_mut();
        }
        if CStr::from_ptr(symbol.cast()).to_bytes() == b"__pthread_get_minstack" {
            // Let's just say we don't support this for now.
            return null_mut();
        }
        #[cfg(target_env = "gnu")]
        if CStr::from_ptr(symbol.cast()).to_bytes() == b"gnu_get_libc_version" {
            return libc::gnu_get_libc_version as *mut c_void;
        }
        #[cfg(any(target_os = "android", target_os = "linux"))]
        if CStr::from_ptr(symbol.cast()).to_bytes() == b"epoll_create1" {
            return libc::epoll_create1 as *mut c_void;
        }
        if CStr::from_ptr(symbol.cast()).to_bytes() == b"pipe2" {
            return libc::pipe2 as *mut c_void;
        }
    }
    unimplemented!("dlsym({:?})", CStr::from_ptr(symbol.cast()))
}

#[no_mangle]
unsafe extern "C" fn dlclose() {
    //libc!(libc::dlclose());
    unimplemented!("dlclose")
}

#[no_mangle]
unsafe extern "C" fn dlerror() {
    //libc!(libc::dlerror());
    unimplemented!("dlerror")
}

#[no_mangle]
unsafe extern "C" fn dlopen() {
    //libc!(libc::dlopen());
    unimplemented!("dlopen")
}

#[no_mangle]
unsafe extern "C" fn sched_yield() -> c_int {
    libc!(libc::sched_yield());

    rustix::process::sched_yield();
    0
}

#[cfg(not(target_os = "wasi"))]
unsafe fn set_cpu(cpu: usize, cpuset: &mut libc::cpu_set_t) {
    libc::CPU_SET(cpu, cpuset)
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn sched_getaffinity(
    pid: c_int,
    cpu_set_size: usize,
    mask: *mut libc::cpu_set_t,
) -> c_int {
    libc!(libc::sched_getaffinity(pid, cpu_set_size, mask.cast()));
    let pid = rustix::process::Pid::from_raw(pid as _);
    match convert_res(rustix::process::sched_getaffinity(pid)) {
        Some(cpu_set) => {
            let mask = &mut *mask;
            (0..cpu_set_size)
                .filter(|&i| cpu_set.is_set(i))
                .for_each(|i| set_cpu(i, mask));
            0
        }
        None => -1,
    }
}

// In Linux, `prctl`'s arguments are described as `unsigned long`, however we
// use pointer types in order to preserve provenance.
#[cfg(any(target_os = "android", target_os = "linux"))]
#[no_mangle]
unsafe extern "C" fn prctl(
    option: c_int,
    arg2: *mut c_void,
    _arg3: *mut c_void,
    _arg4: *mut c_void,
    _arg5: *mut c_void,
) -> c_int {
    libc!(libc::prctl(option, arg2, _arg3, _arg4, _arg5));
    match option {
        libc::PR_SET_NAME => {
            match convert_res(rustix::runtime::set_thread_name(CStr::from_ptr(
                arg2 as *const _,
            ))) {
                Some(()) => 0,
                None => -1,
            }
        }
        libc::PR_GET_PDEATHSIG => match convert_res(rustix::process::parent_process_death_signal())
        {
            Some(signal) => {
                let sig = signal.map(|s| s as u32 as c_int).unwrap_or(0);
                arg2.cast::<c_int>().write(sig);
                0
            }
            None => -1,
        },
        libc::PR_SET_PDEATHSIG => {
            let arg2_i32 =
                match convert_res(i32::try_from(arg2.addr()).map_err(|_| rustix::io::Errno::RANGE))
                {
                    Some(arg2_i32) => arg2_i32,
                    None => return -1,
                };
            // rustix converts any invalid signal to `None`, but only 0 should get mapped
            // to `None`; any other invalid signal is an error
            let sig = if arg2_i32 == 0 {
                None
            } else {
                match convert_res(
                    rustix::process::Signal::from_raw(arg2_i32).ok_or(rustix::io::Errno::RANGE),
                ) {
                    Some(s) => Some(s),
                    None => return -1,
                }
            };
            match convert_res(rustix::process::set_parent_process_death_signal(sig)) {
                Some(()) => 0,
                None => -1,
            }
        }
        libc::PR_GET_DUMPABLE => match convert_res(rustix::process::dumpable_behavior()) {
            Some(dumpable) => dumpable as i32,
            None => -1,
        },
        libc::PR_SET_DUMPABLE => {
            let arg2_i32 =
                match convert_res(i32::try_from(arg2.addr()).map_err(|_| rustix::io::Errno::RANGE))
                {
                    Some(arg2_i32) => arg2_i32,
                    None => return -1,
                };
            let dumpable = match convert_res(rustix::process::DumpableBehavior::try_from(arg2_i32))
            {
                Some(dumpable) => dumpable,
                None => return -1,
            };
            match convert_res(rustix::process::set_dumpable_behavior(dumpable)) {
                Some(()) => 0,
                None => -1,
            }
        }
        _ => unimplemented!("unrecognized prctl op {}", option),
    }
}

#[cfg(target_os = "linux")]
#[no_mangle]
unsafe extern "C" fn pthread_setname_np(
    thread: libc::pthread_t,
    name: *const libc::c_char,
) -> c_int {
    libc!(libc::pthread_setname_np(thread, name));
    match convert_res(rustix::runtime::set_thread_name(CStr::from_ptr(
        name as *const _,
    ))) {
        Some(()) => 0,
        None => -1,
    }
}
