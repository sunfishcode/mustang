use libc::{c_double, c_long, c_ushort};
use rand::distributions::{Standard, Uniform};
use rand::{thread_rng, Rng};

#[no_mangle]
unsafe extern "C" fn drand48() -> c_double {
    //libc!(libc::drand48());

    thread_rng().sample(Standard)
}

#[no_mangle]
unsafe extern "C" fn erand48(_xsubi: &[c_ushort; 3]) -> c_double {
    //libc!(libc::erand48(xsubi));

    unimplemented!("erand48")
}

#[no_mangle]
unsafe extern "C" fn lrand48() -> c_long {
    //libc!(libc::lrand48());

    thread_rng().sample(Uniform::new_inclusive(0, i32::MAX as c_long))
}

#[no_mangle]
unsafe extern "C" fn nrand48(_xsubi: &[c_ushort; 3]) -> c_long {
    //libc!(libc::nrand48(xsubi));

    unimplemented!("nrand48")
}

#[no_mangle]
unsafe extern "C" fn mrand48() -> c_long {
    //libc!(libc::mrand48());

    thread_rng().sample(Uniform::new_inclusive(
        i32::MIN as c_long,
        i32::MAX as c_long,
    ))
}

#[no_mangle]
unsafe extern "C" fn jrand48(_xsubi: &[c_ushort; 3]) -> c_long {
    //libc!(libc::jrand48(xsubi));

    unimplemented!("jrand48")
}

#[no_mangle]
unsafe extern "C" fn srand48(_seedval: c_long) {
    //libc!(libc::srand48(seedval));

    unimplemented!("srand48")
}

#[no_mangle]
unsafe extern "C" fn seed48(_seed16v: &[c_ushort; 3]) -> *mut c_ushort {
    //libc!(libc::seed48(seed16v));

    unimplemented!("seed48")
}

#[no_mangle]
unsafe extern "C" fn lcong48(_param: &[c_ushort; 7]) {
    //libc!(libc::lcong48(param));

    unimplemented!("lcong48")
}
