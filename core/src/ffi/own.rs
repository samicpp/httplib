use crate::ffi::futures::{self, FfiFuture};
use std::{ffi::c_void, sync::atomic::Ordering, ptr};

#[unsafe(no_mangle)]
pub extern "C" fn add_i64(x: i64, y: i64) -> i64 {
    x + y
}


// futures

#[unsafe(no_mangle)]
pub extern "C" fn ffi_future_new(cb: Option<extern "C" fn(*mut c_void)> ) -> *mut FfiFuture{
    Box::into_raw(FfiFuture::new_boxed(cb))
}

#[unsafe(no_mangle)]
pub extern "C" fn ffi_future_state(fut: *const FfiFuture) -> u8{
    unsafe { (*fut).state.load(Ordering::Acquire) }
}

#[unsafe(no_mangle)]
pub extern "C" fn ffi_future_result(fut: *const FfiFuture) -> *mut c_void{
    unsafe {
        if (*fut).state.load(Ordering::Acquire) == futures::READY{
            *(*fut).result.get()
        }
        else {
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ffi_future_take_result(fut: *const FfiFuture) -> *mut c_void{
    unsafe {
        if (*fut).state.load(Ordering::Acquire) == futures::READY{
            let rptr = (*fut).result.get();
            let result = *rptr;
            *rptr = ptr::null_mut();
            result
        }
        else {
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ffi_future_cancel(fut: *const FfiFuture) {
    unsafe { (*fut).cancel() }
}

#[unsafe(no_mangle)]
pub extern "C" fn ffi_future_complete(fut: *const FfiFuture, result: *mut c_void) {
    unsafe { (*fut).complete(result) }
}

#[unsafe(no_mangle)]
pub extern "C" fn ffi_future_free(fut: *mut FfiFuture) {
    unsafe { drop(Box::from_raw(fut)) }
}
