use core::convert::TryInto;
use core::mem::transmute;
use core::ptr::null_mut;

use errno::{set_errno, Errno};
use libc::{c_int, c_void};
use memoffset::offset_of;

use super::CScapeDir;

#[no_mangle]
unsafe extern "C" fn readdir64_r(
    dir: *mut c_void,
    entry: *mut libc::dirent64,
    ptr: *mut *mut libc::dirent64,
) -> c_int {
    libc!(libc::readdir64_r(dir.cast(), entry, ptr));

    let mustang_dir = dir.cast::<CScapeDir>();
    let dir = &mut (*mustang_dir).dir;
    match dir.read() {
        None => {
            *ptr = null_mut();
            0
        }
        Some(Ok(e)) => {
            let file_type = match e.file_type() {
                rustix::fs::FileType::RegularFile => libc::DT_REG,
                rustix::fs::FileType::Directory => libc::DT_DIR,
                rustix::fs::FileType::Symlink => libc::DT_LNK,
                rustix::fs::FileType::Fifo => libc::DT_FIFO,
                rustix::fs::FileType::Socket => libc::DT_SOCK,
                rustix::fs::FileType::CharacterDevice => libc::DT_CHR,
                rustix::fs::FileType::BlockDevice => libc::DT_BLK,
                rustix::fs::FileType::Unknown => libc::DT_UNKNOWN,
            };
            *entry = libc::dirent64 {
                d_ino: e.ino(),
                d_off: 0, // We don't implement `seekdir` yet anyway.
                d_reclen: (offset_of!(libc::dirent64, d_name) + e.file_name().to_bytes().len() + 1)
                    .try_into()
                    .unwrap(),
                d_type: file_type,
                d_name: [0; 256],
            };
            let len = core::cmp::min(256, e.file_name().to_bytes().len());
            (*entry).d_name[..len].copy_from_slice(transmute(e.file_name().to_bytes()));
            *ptr = entry;
            0
        }
        Some(Err(err)) => err.raw_os_error(),
    }
}

#[no_mangle]
unsafe extern "C" fn readdir64(dir: *mut libc::DIR) -> *mut libc::dirent64 {
    libc!(libc::readdir64(dir.cast()));

    let mustang_dir = dir.cast::<CScapeDir>();
    let dir = &mut (*mustang_dir).dir;
    match dir.read() {
        None => null_mut(),
        Some(Ok(e)) => {
            let file_type = match e.file_type() {
                rustix::fs::FileType::RegularFile => libc::DT_REG,
                rustix::fs::FileType::Directory => libc::DT_DIR,
                rustix::fs::FileType::Symlink => libc::DT_LNK,
                rustix::fs::FileType::Fifo => libc::DT_FIFO,
                rustix::fs::FileType::Socket => libc::DT_SOCK,
                rustix::fs::FileType::CharacterDevice => libc::DT_CHR,
                rustix::fs::FileType::BlockDevice => libc::DT_BLK,
                rustix::fs::FileType::Unknown => libc::DT_UNKNOWN,
            };
            (*mustang_dir).storage.dirent64 = libc::dirent64 {
                d_ino: e.ino(),
                d_off: 0, // We don't implement `seekdir` yet anyway.
                d_reclen: (offset_of!(libc::dirent64, d_name) + e.file_name().to_bytes().len() + 1)
                    .try_into()
                    .unwrap(),
                d_type: file_type,
                d_name: [0; 256],
            };
            let len = core::cmp::min(256, e.file_name().to_bytes().len());
            (*mustang_dir).storage.dirent64.d_name[..len]
                .copy_from_slice(transmute(e.file_name().to_bytes()));
            &mut (*mustang_dir).storage.dirent64
        }
        Some(Err(err)) => {
            set_errno(Errno(err.raw_os_error()));
            null_mut()
        }
    }
}


#[no_mangle]
unsafe extern "C" fn readdir(dir: *mut libc::DIR) -> *mut libc::dirent {
    libc!(libc::readdir(dir.cast()));

    let mustang_dir = dir.cast::<CScapeDir>();
    let dir = &mut (*mustang_dir).dir;
    match dir.read() {
        None => null_mut(),
        Some(Ok(e)) => {
            let file_type = match e.file_type() {
                rustix::fs::FileType::RegularFile => libc::DT_REG,
                rustix::fs::FileType::Directory => libc::DT_DIR,
                rustix::fs::FileType::Symlink => libc::DT_LNK,
                rustix::fs::FileType::Fifo => libc::DT_FIFO,
                rustix::fs::FileType::Socket => libc::DT_SOCK,
                rustix::fs::FileType::CharacterDevice => libc::DT_CHR,
                rustix::fs::FileType::BlockDevice => libc::DT_BLK,
                rustix::fs::FileType::Unknown => libc::DT_UNKNOWN,
            };
            
            let result: Result<(), core::num::TryFromIntError> = try {
                (*mustang_dir).storage.dirent = libc::dirent {
                    d_ino: e.ino().try_into()?,
                    d_off: 0, // We don't implement `seekdir` yet anyway.
                    d_reclen: (offset_of!(libc::dirent64, d_name) + e.file_name().to_bytes().len() + 1)
                        .try_into()
                        .unwrap(),
                    d_type: file_type,
                    d_name: [0; 256],
                };
            };

            match result {
                Err(_) => {
                    set_errno(Errno(libc::EOVERFLOW));
                    return null_mut();
                },
                Ok(()) => {},
            }
            

            let len = core::cmp::min(256, e.file_name().to_bytes().len());
            (*mustang_dir).storage.dirent.d_name[..len]
                .copy_from_slice(transmute(e.file_name().to_bytes()));
            &mut (*mustang_dir).storage.dirent
        }
        Some(Err(err)) => {
            set_errno(Errno(err.raw_os_error()));
            null_mut()
        }
    }
}

#[no_mangle]
unsafe extern "C" fn rewinddir() {
    //libc!(libc::rewinddir());
    unimplemented!("rewinddir")
}

#[no_mangle]
unsafe extern "C" fn scandir() {
    //libc!(libc::scandir());
    unimplemented!("scandir")
}

#[no_mangle]
unsafe extern "C" fn seekdir() {
    //libc!(libc::seekdir());
    unimplemented!("seekdir")
}

#[no_mangle]
unsafe extern "C" fn telldir() {
    //libc!(libc::telldir());
    unimplemented!("telldir")
}
