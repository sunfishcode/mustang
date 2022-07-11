mod ctype_b_loc;
mod ctype_tolower_loc;
mod ctype_toupper_loc;

#[repr(transparent)]
#[derive(Copy, Clone)]
struct SyncWrapper<T>(T);
unsafe impl <T> Sync for SyncWrapper<T> {}

#[no_mangle]
extern "C" fn __ctype_get_mb_cur_max() -> libc::size_t {
    unimplemented!()
}