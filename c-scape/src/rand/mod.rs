use libc::{c_int, c_uint};

use rand::Rng;
use rand_core::SeedableRng;
use rand_pcg::Pcg32;

use origin::sync::Mutex;

#[cfg(test)]
static_assertions::assert_eq_size!(c_uint, u32);

// We use PCG32 here, which takes just 16 bytes and offers us a relatively nice
// non-cryptographic RNG for most applications.
// TODO: Remove the option once Pcg32 has seedable const fn things
static STATE: Mutex<Option<Pcg32>> = Mutex::new(None);

#[no_mangle]
unsafe extern "C" fn rand() -> c_int {
    libc!(libc::rand());

    // POSIX requires that if the user hasn't initialized the RNG, it behaves
    // as-if srand(1) was called.
    let mut guard = STATE.lock();
    if *guard == None {
        internal_seed(&mut guard, 1);
    }

    // Sample using a uniform distribution
    (*guard)
        .as_mut()
        .unwrap()
        .gen_range(0..libc::RAND_MAX) as c_int
}

#[no_mangle]
unsafe extern "C" fn srand(seed: c_uint) {
    libc!(libc::srand(seed));

    internal_seed(&mut STATE.lock(), seed);
}

fn internal_seed(state: &mut Option<Pcg32>, seed: c_uint) {
    *state = Some(Pcg32::seed_from_u64(u64::from(seed)));
}

// `rand_r` gives us at most 32 bits of internal state, which is quite frankly
// terrible. Still, do the best we can.
//
// The algorithm used for this is Mulberry32, which
// is under the CC0 license.
// https://gist.github.com/tommyettinger/46a874533244883189143505d203312c
#[no_mangle]
unsafe extern "C" fn rand_r(seed: *mut c_uint) -> c_int {
    // libc!(libc::rand_r(seed));

    let mut z = *seed + 0x6D2B79F5;
    *seed = z;

    z = (z ^ (z >> 15)) * (z | 1);
    z ^= z + (z ^ (z >> 7)) * (z | 61);
    return (z ^ (z >> 14)) as c_int;
}
