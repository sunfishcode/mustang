// TODO: Re-enable test_simd when core_simd works on Rust nightly.
//#![feature(portable_simd)]
//#![feature(asm)]

mustang::can_run_this!();

fn main() {
    // TODO: Re-enable test_simd when core_simd works on Rust nightly.
    /*
        use core_simd::*;
        let mut a = f32x4::splat(2.0);
        unsafe { asm!("# {}", in(reg) &mut a) };
        assert_eq!(a, f32x4::splat(2.0));
        assert_eq!(&a as *const _ as usize & 0xf, 0);
    */
}
