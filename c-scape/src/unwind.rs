//! Stub libunwind implementation on platforms where we don't have real
//! unwind support.
//!
//! Entirely `unimplemented!`. Don't panic.

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn _Unwind_Backtrace() {
    unimplemented!("_Unwind_Backtrace")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn _Unwind_DeleteException() {
    unimplemented!("_Unwind_DeleteException")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn _Unwind_GetDataRelBase() {
    unimplemented!("_Unwind_GetDataRelBase")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn _Unwind_GetIP() {
    unimplemented!("_Unwind_GetIP")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn _Unwind_GetIPInfo() {
    unimplemented!("_Unwind_GetIPInfo")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn _Unwind_GetLanguageSpecificData() {
    unimplemented!("_Unwind_GetLanguageSpecificData")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn _Unwind_GetRegionStart() {
    unimplemented!("_Unwind_GetRegionStart")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn _Unwind_GetTextRelBase() {
    unimplemented!("_Unwind_GetTextRelBase")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn _Unwind_RaiseException() {
    unimplemented!("_Unwind_RaiseException")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn _Unwind_Resume() {
    unimplemented!("_Unwind_Resume")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn _Unwind_SetGR() {
    unimplemented!("_Unwind_SetGR")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn _Unwind_SetIP() {
    unimplemented!("_Unwind_SetIP")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn _Unwind_FindEnclosingFunction() {
    unimplemented!("_Unwind_FindEnclosingFunction")
}

#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
unsafe extern "C" fn _Unwind_GetCFA() {
    unimplemented!("_Unwind_GetCFA")
}

/// Ensure that this module is linked in.
#[inline(never)]
#[link_section = ".text.__mustang"]
#[no_mangle]
#[cold]
unsafe extern "C" fn __mustang_c_scape__unwind() {}
