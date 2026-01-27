use core::slice;
use std::{ffi::{CStr, c_void}, net::SocketAddr, ptr};

use http::shared::HttpSocket;
use httprs_core::ffi::{futures::FfiFuture, own::{FfiSlice, RT}};

use crate::servers::{DynHttpSocket, Server, TcpServer};


#[repr(C)]
pub struct FfiBundle{
    pub http: DynHttpSocket,
    pub addr: SocketAddr,
}


#[unsafe(no_mangle)]
pub extern "C" fn server_new_tcp(fut: *mut FfiFuture, string: *mut i8){
    unsafe {
        let addr = CStr::from_ptr(string).to_string_lossy().to_string();
        let fut = Box::from_raw(fut);

        RT.get().unwrap().spawn(async move {
            match TcpServer::new(addr).await{
                Ok(r) => {
                    let boxed: Box<dyn Server> = Box::new(r);
                    let ptr = Box::into_raw(boxed);
                    fut.complete(ptr as *mut c_void)
                },
                Err(_) => (*fut).cancel(),
            }
            let _ = Box::into_raw(fut);
        });
    }
}

#[allow(improper_ctypes_definitions)]
#[unsafe(no_mangle)]
pub extern "C" fn server_accept(fut: *mut FfiFuture, server: *mut (dyn Server + Send)){
    unsafe {
        let mut server = Box::from_raw(server);
        let fut = Box::from_raw(fut);

        RT.get().unwrap().spawn(async move {
            match server.accept().await{
                Ok((addr, http)) => {
                    // let boxed = Box::new(http);
                    // let ptr = Box::into_raw(boxed);

                    let ffi = FfiBundle {
                        http,
                        addr,
                    };

                    fut.complete(Box::into_raw(Box::new(ffi)) as *mut c_void)
                },
                Err(_) => (*fut).cancel(),
            }

            let _ = Box::into_raw(server);
            let _ = Box::into_raw(fut);
        });
    }
}


#[unsafe(no_mangle)]
pub extern "C" fn http_close(fut: *mut FfiFuture, ffi: *mut FfiBundle, len: usize, ptr: *const u8){
    unsafe{
        let mut ffi = Box::from_raw(ffi);
        let fut = Box::from_raw(fut);

        let buff = slice::from_raw_parts(ptr, len);
        RT.get().unwrap().spawn(async move{
            match ffi.http.close(buff).await{
                Ok(_) => fut.complete(ptr::null_mut()),
                Err(_) => fut.cancel(),
            }

            let _ = Box::into_raw(ffi);
            let _ = Box::into_raw(fut);
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn get_addr_str(ffi: *mut FfiBundle) -> FfiSlice{
    unsafe{
        let astr = (*ffi).addr.to_string();
        let mut bytes = astr.into_bytes();
        let ptr = bytes.as_mut_ptr();
        let len = bytes.len();
        std::mem::forget(bytes);

        FfiSlice { ptr, len }
    }
}