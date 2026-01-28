use tokio::runtime::Runtime;
use crate::ffi::futures::{self, FfiFuture};
use core::slice;
use std::{ffi::c_void, ptr, sync::{OnceLock, atomic::Ordering}};


// tokio

pub static RT: OnceLock<Runtime> = OnceLock::new();

#[unsafe(no_mangle)]
pub extern "C" fn init_rt() -> bool{
    if let Ok(rt) = tokio::runtime::Builder::new_multi_thread().enable_all().build(){
        RT.set(rt).is_ok()
    }
    else{
        false
    }
}

#[allow(static_mut_refs)]
#[unsafe(no_mangle)]
pub extern "C" fn has_init() -> bool{
    RT.get().is_some()
}


// futures

#[unsafe(no_mangle)]
pub extern "C" fn ffi_future_new(cb: Option<extern "C" fn(*mut c_void, *mut c_void)>, userdata: *mut c_void) -> *mut FfiFuture{
    Box::into_raw(FfiFuture::new_boxed(cb, userdata))
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

#[unsafe(no_mangle)]
pub extern "C" fn ffi_future_await(fut: *mut FfiFuture) {
    unsafe {
        let rfut = (*fut).to_future();
        RT.get().unwrap().block_on(async move {
            rfut.await;
        })
    }
}


// slice

#[repr(C)]
#[derive(Debug)]
pub struct FfiSlice{
    pub owned: bool,
    pub len: usize,
    pub cap: usize,
    pub ptr: *const u8,
}

impl FfiSlice{
    pub fn from_string(string: String) -> Self{
        let bytes = string.into_bytes();
        let ptr = bytes.as_ptr();
        let len = bytes.len();
        let cap = bytes.capacity();
        std::mem::forget(bytes);

        Self {
            owned: true,
            len,
            cap,
            ptr,
        }
    }
    pub fn from_vec(vec: Vec<u8>) -> Self{
        let ptr = vec.as_ptr();
        let len = vec.len();
        let cap = vec.capacity();
        std::mem::forget(vec);

        Self {
            owned: true,
            len,
            cap,
            ptr,
        }
    }
    pub fn from_str(str_slice: &str) -> Self{
        let ptr = str_slice.as_ptr();
        let len = str_slice.len();

        Self {
            owned: false,
            len,
            ptr,
            cap: len,
        }
    }
    pub fn from_buf(slice: &[u8]) -> Self{
        let ptr = slice.as_ptr();
        let len = slice.len();

        Self {
            owned: false,
            len,
            ptr,
            cap: len,
        }
    }

    pub fn empty() -> Self{
        Self { len: 0, cap: 0, ptr: ptr::null(), owned: false }
    }

    pub fn free(self) {
        if self.owned && self.ptr != ptr::null(){
            drop(self.to_vec());
        }
    }
    pub fn to_string(self) -> Option<String>{
        if !self.owned {
            None
        }
        else {
            unsafe { Some(String::from_raw_parts(self.ptr as *mut u8, self.len, self.cap)) }
        }
    }
    pub fn to_vec(self) -> Option<Vec<u8>>{
        if !self.owned { None }
        else{
            unsafe { Some(Vec::from_raw_parts(self.ptr as *mut u8, self.len, self.cap)) }
        }
    }
    pub fn as_bytes(&self) -> &[u8]{
        unsafe { slice::from_raw_parts(self.ptr, self.len) }
    }
    pub fn as_str(&self) -> std::borrow::Cow<'_, str>{
        String::from_utf8_lossy(self.as_bytes())
    }
}

unsafe impl Sync for FfiSlice{}
unsafe impl Send for FfiSlice{}

#[unsafe(no_mangle)]
pub extern "C" fn free_slice(slice: FfiSlice) {
    slice.free();
}

// test

#[unsafe(no_mangle)]
pub extern "C" fn add_i64(x: i64, y: i64) -> i64 {
    x + y
}