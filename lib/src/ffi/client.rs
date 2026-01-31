use std::ffi::{CStr, c_void};

use http::http1::client::Http1Request;
use httprs_core::ffi::{futures::FfiFuture, own::RT};

use crate::{DynStream, clients::{DynHttpRequest, tcp_connect as ntcpconn}, errno::IO_ERROR};



#[unsafe(no_mangle)]
pub extern "C" fn tcp_connect(fut: *mut FfiFuture, addr: *mut i8){
    unsafe{
        let addr = CStr::from_ptr(addr).to_string_lossy().to_string();
        let fut = Box::from_raw(fut);

        RT.get().unwrap().spawn(async move{
            match ntcpconn(addr).await {
                Ok(tcp) => fut.complete(Box::into_raw(Box::new(tcp)) as *mut c_void),
                Err(e) => fut.cancel_with_err(IO_ERROR, e.to_string().into()),
            }
            let _ = Box::into_raw(fut);
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http1_request_new(stream: *mut DynStream, bufsize: usize) -> *mut DynHttpRequest{
    unsafe{
        let stream = Box::from_raw(stream);
        let dreq = Http1Request::new(stream.to_stream(), bufsize).into();
        Box::into_raw(Box::new(dreq))
    }
}
