#![feature(portable_simd)]

mustang::can_run_this!();

//use core::arch::asm;

fn main() {
    // TODO: Reenable this when the crate compiles on nightly.
    /*
    use core_simd::simd::*;
    let mut a = f32x4::splat(2.0);
    unsafe { asm!("# {}", in(reg) &mut a) };
    assert_eq!(a, f32x4::splat(2.0));
    assert_eq!(&a as *const _ as usize & 0xf, 0);
    */
}
