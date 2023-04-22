use libc::{c_int, c_long, c_ulong};

#[no_mangle]
unsafe extern "C" fn get_nprocs_conf() -> c_int {
    //libc!(libc::get_nprocs_conf());

    match std::thread::available_parallelism() {
        Ok(n) => n.get().try_into().unwrap_or(c_int::MAX),
        Err(_) => 1,
    }
}

#[no_mangle]
unsafe extern "C" fn get_nprocs() -> c_int {
    //libc!(libc::get_nprocs());

    get_nprocs_conf()
}

#[no_mangle]
unsafe extern "C" fn get_phys_pages() -> c_long {
    //libc!(libc::get_phys_pages());

    let info = rustix::process::sysinfo();
    let mem_unit = if info.mem_unit == 0 {
        1
    } else {
        info.mem_unit as c_ulong
    };

    (info.totalram * mem_unit / rustix::param::page_size() as c_ulong)
        .try_into()
        .unwrap_or(c_long::MAX)
}

#[no_mangle]
unsafe extern "C" fn get_avphys_pages() -> c_long {
    //libc!(libc::get_avphys_pages());

    let info = rustix::process::sysinfo();
    let mem_unit = if info.mem_unit == 0 {
        1
    } else {
        info.mem_unit as c_ulong
    };

    ((info.freeram + info.bufferram) * mem_unit / rustix::param::page_size() as c_ulong)
        .try_into()
        .unwrap_or(c_long::MAX)
}
