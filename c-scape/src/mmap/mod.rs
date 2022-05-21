use rustix::fd::BorrowedFd;
use rustix::mm::{MapFlags, MprotectFlags, MremapFlags, ProtFlags};

use core::ffi::c_void;
use libc::{c_int, off64_t, off_t};

use crate::convert_res;

#[no_mangle]
unsafe extern "C" fn mmap(
    addr: *mut c_void,
    length: usize,
    prot: c_int,
    flags: c_int,
    fd: c_int,
    offset: off_t,
) -> *mut c_void {
    libc!(libc::mmap(addr, length, prot, flags, fd, offset));

    mmap64(addr, length, prot, flags, fd, offset as off64_t)
}

#[no_mangle]
unsafe extern "C" fn mmap64(
    addr: *mut c_void,
    length: usize,
    prot: c_int,
    flags: c_int,
    fd: c_int,
    offset: off64_t,
) -> *mut c_void {
    libc!(libc::mmap64(addr, length, prot, flags, fd, offset));

    let anon = flags & libc::MAP_ANONYMOUS == libc::MAP_ANONYMOUS;
    let prot = ProtFlags::from_bits(prot as _).unwrap();
    let flags = MapFlags::from_bits((flags & !libc::MAP_ANONYMOUS) as _).unwrap();
    match convert_res(if anon {
        rustix::mm::mmap_anonymous(addr, length, prot, flags)
    } else {
        rustix::mm::mmap(
            addr,
            length,
            prot,
            flags,
            BorrowedFd::borrow_raw(fd),
            offset as _,
        )
    }) {
        Some(ptr) => ptr,
        None => libc::MAP_FAILED,
    }
}

#[no_mangle]
unsafe extern "C" fn munmap(ptr: *mut c_void, len: usize) -> c_int {
    libc!(libc::munmap(ptr, len));

    match convert_res(rustix::mm::munmap(ptr, len)) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn mremap(
    old_address: *mut c_void,
    old_size: usize,
    new_size: usize,
    flags: c_int,
    mut args: ...
) -> *mut c_void {
    if (flags & libc::MREMAP_FIXED) == libc::MREMAP_FIXED {
        let new_address = args.arg::<*mut c_void>();
        libc!(libc::mremap(
            old_address,
            old_size,
            new_size,
            flags,
            new_address
        ));

        let flags = flags & !libc::MREMAP_FIXED;
        let flags = MremapFlags::from_bits(flags as _).unwrap();
        match convert_res(rustix::mm::mremap_fixed(
            old_address,
            old_size,
            new_size,
            flags,
            new_address,
        )) {
            Some(new_address) => new_address,
            None => libc::MAP_FAILED,
        }
    } else {
        libc!(libc::mremap(old_address, old_size, new_size, flags));

        let flags = MremapFlags::from_bits(flags as _).unwrap();
        match convert_res(rustix::mm::mremap(old_address, old_size, new_size, flags)) {
            Some(new_address) => new_address,
            None => libc::MAP_FAILED,
        }
    }
}

#[no_mangle]
unsafe extern "C" fn mprotect(addr: *mut c_void, length: usize, prot: c_int) -> c_int {
    libc!(libc::mprotect(addr, length, prot));

    let prot = MprotectFlags::from_bits(prot as _).unwrap();
    match convert_res(rustix::mm::mprotect(addr, length, prot)) {
        Some(()) => 0,
        None => -1,
    }
}
