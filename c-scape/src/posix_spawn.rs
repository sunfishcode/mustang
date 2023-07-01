use libc::{c_int, c_void};

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn posix_spawnp() {
    //libc!(libc::posix_spawnp());
    unimplemented!("posix_spawnp");
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn posix_spawnattr_destroy(_ptr: *const c_void) -> c_int {
    //libc!(libc::posix_spawn_spawnattr_destroy(ptr));
    rustix::io::write(
        rustix::stdio::stderr(),
        b"unimplemented: posix_spawn_spawnattr_destroy\n",
    )
    .ok();
    0
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn posix_spawnattr_init(_ptr: *const c_void) -> c_int {
    //libc!(libc::posix_spawnattr_init(ptr));
    rustix::io::write(
        rustix::stdio::stderr(),
        b"unimplemented: posix_spawnattr_init\n",
    )
    .ok();
    0
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn posix_spawnattr_setflags() {
    //libc!(libc::posix_spawnattr_setflags());
    unimplemented!("posix_spawnattr_setflags")
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn posix_spawnattr_setsigdefault() {
    //libc!(libc::posix_spawnattr_setsigdefault());
    unimplemented!("posix_spawnattr_setsigdefault")
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn posix_spawnattr_setsigmask() {
    //libc!(libc::posix_spawnattr_setsigmask());
    unimplemented!("posix_spawnsetsigmask")
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn posix_spawnattr_setpgroup(_ptr: *mut c_void, _pgroup: c_int) -> c_int {
    //libc!(libc::posix_spawnattr_setpgroup(ptr, pgroup));
    unimplemented!("posix_spawnattr_setpgroup")
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn posix_spawn_file_actions_adddup2() {
    //libc!(libc::posix_spawn_file_actions_adddup2());
    unimplemented!("posix_spawn_file_actions_adddup2")
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn posix_spawn_file_actions_addchdir_np() {
    //libc!(libc::posix_spawn_file_actions_addchdir_np());
    unimplemented!("posix_spawn_file_actions_addchdir_np")
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn posix_spawn_file_actions_destroy(_ptr: *const c_void) -> c_int {
    //libc!(libc::posix_spawn_file_actions_destroy(ptr));
    rustix::io::write(
        rustix::stdio::stderr(),
        b"unimplemented: posix_spawn_file_actions_destroy\n",
    )
    .ok();
    0
}

#[cfg(not(target_os = "wasi"))]
#[no_mangle]
unsafe extern "C" fn posix_spawn_file_actions_init(_ptr: *const c_void) -> c_int {
    //libc!(libc::posix_spawn_file_actions_init(ptr));
    rustix::io::write(
        rustix::stdio::stderr(),
        b"unimplemented: posix_spawn_file_actions_init\n",
    )
    .ok();
    0
}
