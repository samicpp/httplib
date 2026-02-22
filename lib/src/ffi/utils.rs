use core::ffi::c_void;
use std::ptr;

use httprs_core::ffi::{futures::FfiFuture, own::{FfiSlice, RT}};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{DynStream, errno::{Errno, TYPE_ERR}};


pub fn heap_ptr<T>(thing: T) -> *mut T{
    Box::into_raw(Box::new(thing))
}
pub fn heap_void_ptr<T>(thing: T) -> *mut c_void {
    Box::into_raw(Box::new(thing)) as *mut c_void
}
pub fn heap_const_ptr<T>(thing: T) -> *const T{
    Box::into_raw(Box::new(thing))
}


#[repr(C)]
#[derive(Debug)]
pub struct FfiDuoStream {
    pub one: *mut DynStream, // idk
    pub two: *mut DynStream, // 
}


#[unsafe(no_mangle)]
pub extern "C" fn create_duplex(bufsize: usize) -> FfiDuoStream {
    let duo = tokio::io::duplex(bufsize);
    FfiDuoStream {
        one: heap_ptr(duo.0.into()),
        two: heap_ptr(duo.1.into()),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn tcp_peek(fut: *mut FfiFuture, ffi: *mut DynStream, buf: *mut FfiSlice){
    unsafe {
        let ffi = &mut *ffi;
        let fut = &*fut;
        let buf = (*buf).as_bytes_mut();

        RT.get().unwrap().spawn(async move {
            if let DynStream::Tcp(tcp) = ffi {
                match tcp.peek(buf).await {
                    Ok(size) => fut.complete(heap_void_ptr(size)),
                    Err(e) => fut.cancel_with_err(e.get_errno(), e.to_string().into()),
                }
            }
            else{
                fut.cancel_with_err(TYPE_ERR, "socket not tcp".into())
            }
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn stream_read(fut: *mut FfiFuture, stream: *mut DynStream, buf: *mut FfiSlice){
    unsafe {
        let stream = &mut *stream;
        let fut = &*fut;
        let buf = (*buf).as_bytes_mut();

        RT.get().unwrap().spawn(async move {
            match stream.read(buf).await {
                Ok(size) => fut.complete(heap_void_ptr(size)),
                Err(e) => fut.cancel_with_err(e.get_errno(), e.to_string().into()),
            }
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn stream_write(fut: *mut FfiFuture, stream: *mut DynStream, buf: *mut FfiSlice){
    unsafe {
        let stream = &mut *stream;
        let fut = &*fut;
        let buf = (*buf).as_bytes_mut();

        RT.get().unwrap().spawn(async move {
            match stream.write(buf).await {
                Ok(size) => fut.complete(heap_void_ptr(size)),
                Err(e) => fut.cancel_with_err(e.get_errno(), e.to_string().into()),
            }
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn stream_write_all(fut: *mut FfiFuture, stream: *mut DynStream, buf: *mut FfiSlice){
    unsafe {
        let stream = &mut *stream;
        let fut = &*fut;
        let buf = (*buf).as_bytes_mut();

        RT.get().unwrap().spawn(async move {
            match stream.write_all(buf).await {
                Ok(_) => fut.complete(ptr::null_mut()),
                Err(e) => fut.cancel_with_err(e.get_errno(), e.to_string().into()),
            }
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn stream_free(stream: *mut DynStream){
    unsafe {
        drop(Box::from_raw(stream))
    }
}

