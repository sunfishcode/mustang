use crate::convert_res;
use errno::{set_errno, Errno};
use libc::c_int;

#[no_mangle]
unsafe extern "C" fn getgroups(num: c_int, groups: *mut libc::gid_t) -> c_int {
    libc!(libc::getgroups(num, groups));

    let returned_groups = match convert_res(rustix::process::getgroups()) {
        Some(returned_groups) => returned_groups,
        None => return -1,
    };

    if num == 0 {
        return match returned_groups.len().try_into() {
            Ok(converted) => converted,
            Err(_) => {
                set_errno(Errno(libc::ENOMEM));
                -1
            }
        };
    }

    let mut num_out = 0;
    let mut groups = groups;

    for group in returned_groups {
        if num_out == num {
            set_errno(Errno(libc::EINVAL));
            return -1;
        }
        num_out += 1;
        groups.write(group.as_raw());
        groups = groups.add(1);
    }

    num_out
}

#[cfg(any(target_os = "android", target_os = "linux"))]
#[no_mangle]
unsafe extern "C" fn setgroups() {
    //libc!(libc::setgroups());
    unimplemented!("setgroups")
}
