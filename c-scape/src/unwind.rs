//! Libunwind.
//!
//! Entirely `unimplemented!`. Don't panic.

#[used]
pub static USED: unsafe extern "C" fn() = _Unwind_Backtrace;

#[no_mangle]
pub unsafe extern "C" fn _Unwind_Backtrace() {
    unimplemented!("_Unwind_Backtrace")
}

#[no_mangle]
pub unsafe extern "C" fn _Unwind_DeleteException() {
    unimplemented!("_Unwind_DeleteException")
}

#[no_mangle]
pub unsafe extern "C" fn _Unwind_GetDataRelBase() {
    unimplemented!("_Unwind_GetDataRelBase")
}

#[no_mangle]
pub unsafe extern "C" fn _Unwind_GetIP() {
    unimplemented!("_Unwind_GetIP")
}

#[no_mangle]
pub unsafe extern "C" fn _Unwind_GetIPInfo() {
    unimplemented!("_Unwind_GetIPInfo")
}

#[no_mangle]
pub unsafe extern "C" fn _Unwind_GetLanguageSpecificData() {
    unimplemented!("_Unwind_GetLanguageSpecificData")
}

#[no_mangle]
pub unsafe extern "C" fn _Unwind_GetRegionStart() {
    unimplemented!("_Unwind_GetRegionStart")
}

#[no_mangle]
pub unsafe extern "C" fn _Unwind_GetTextRelBase() {
    unimplemented!("_Unwind_GetTextRelBase")
}

#[no_mangle]
pub unsafe extern "C" fn _Unwind_RaiseException() {
    unimplemented!("_Unwind_RaiseException")
}

#[no_mangle]
pub unsafe extern "C" fn _Unwind_Resume() {
    unimplemented!("_Unwind_Resume")
}

#[no_mangle]
pub unsafe extern "C" fn _Unwind_SetGR() {
    unimplemented!("_Unwind_SetGR")
}

#[no_mangle]
pub unsafe extern "C" fn _Unwind_SetIP() {
    unimplemented!("_Unwind_SetIP")
}

#[no_mangle]
pub unsafe extern "C" fn _Unwind_FindEnclosingFUnction() {
    unimplemented!("_Unwind_FindEnclosingFUnction")
}

#[no_mangle]
pub unsafe extern "C" fn _Unwind_GetCFA() {
    unimplemented!("_Unwind_GetCFA")
}

#[no_mangle]
pub unsafe extern "C" fn _Unwind_FindEnclosingFunction() {
    unimplemented!("_Unwind_FindEnclosingFunction")
}
