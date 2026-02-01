use core::slice;
use std::{ffi::{CStr, c_void}, ptr};

use http::{http1::client::Http1Request, shared::{HttpMethod, HttpRequest, HttpType}};
use httprs_core::ffi::{futures::FfiFuture, own::{FfiSlice, RT}};

use crate::{DynStream, clients::{DynHttpRequest, tcp_connect as ntcpconn, tls_upgrade, tls_upgrade_no_verification}, errno::{ERROR, IO_ERROR}, ffi::{const_enums::methods, server::FfiHeaderPair}};



#[unsafe(no_mangle)]
pub extern "C" fn tcp_connect(fut: *mut FfiFuture, addr: *mut i8){
    unsafe{
        let addr = CStr::from_ptr(addr).to_string_lossy().to_string();
        let fut = Box::from_raw(fut);

        RT.get().unwrap().spawn(async move{
            match ntcpconn(addr).await {
                Ok(tcp) => fut.complete(Box::into_raw(Box::new(DynStream::from(tcp))) as *mut c_void),
                Err(e) => fut.cancel_with_err(IO_ERROR, e.to_string().into()),
            }
            let _ = Box::into_raw(fut);
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn tcp_tls_connect(fut: *mut FfiFuture, addr: *mut i8, domain: *mut i8, len: usize, alpns: *const FfiSlice){
    unsafe{
        let addr = CStr::from_ptr(addr).to_string_lossy().to_string();
        let domain = CStr::from_ptr(domain).to_string_lossy().to_string();
        let fut = Box::from_raw(fut);
        let alpns = slice::from_raw_parts(alpns, len).iter().map(|s| s.as_bytes().to_vec()).collect();

        RT.get().unwrap().spawn(async move{
            match ntcpconn(addr).await {
                Ok(tcp) => match tls_upgrade(tcp, domain, alpns).await {
                    Ok(tls) => fut.complete(Box::into_raw(Box::new(DynStream::from(tls))) as *mut c_void),
                    Err(e) => fut.cancel_with_err(IO_ERROR, e.to_string().into()),
                },
                Err(e) => fut.cancel_with_err(IO_ERROR, e.to_string().into()),
            }
            let _ = Box::into_raw(fut);
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn tcp_tls_connect_unverified(fut: *mut FfiFuture, addr: *mut i8, domain: *mut i8, len: usize, alpns: *const FfiSlice){
    unsafe{
        let addr = CStr::from_ptr(addr).to_string_lossy().to_string();
        let domain = CStr::from_ptr(domain).to_string_lossy().to_string();
        let fut = Box::from_raw(fut);
        let alpns = slice::from_raw_parts(alpns, len).iter().map(|s| s.as_bytes().to_vec()).collect();

        RT.get().unwrap().spawn(async move{
            match ntcpconn(addr).await {
                Ok(tcp) => match tls_upgrade_no_verification(tcp, domain, alpns).await {
                    Ok(tls) => fut.complete(Box::into_raw(Box::new(DynStream::from(tls))) as *mut c_void),
                    Err(e) => fut.cancel_with_err(IO_ERROR, e.to_string().into()),
                },
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

#[unsafe(no_mangle)]
pub extern "C" fn http_req_get_type(http: *mut DynHttpRequest) -> u8{
    unsafe {
        match (*http).get_type() {
            HttpType::Http1 => 1,
            HttpType::Http2 => 1,
            HttpType::Http3 => 1,
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http_req_set_header(req: *mut DynHttpRequest, pair: FfiHeaderPair){
    unsafe{
        let name = pair.nam.as_str();
        let value = pair.val.as_str();

        (*req).set_header(&name, &value);
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_req_add_header(req: *mut DynHttpRequest, pair: FfiHeaderPair){
    unsafe{
        let name = pair.nam.as_str();
        let value = pair.val.as_str();

        (*req).add_header(&name, &value);
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_req_del_header(req: *mut DynHttpRequest, name: FfiSlice){
    unsafe{
        let name = name.as_str();
        let _ = (*req).del_header(&name);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http_req_set_method_str(req: *mut DynHttpRequest, method: FfiSlice){
    unsafe{
        let meth = method.as_str().as_ref().into();
        (*req).set_method(meth);
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_req_set_method_byte(req: *mut DynHttpRequest, method: u8){
    unsafe{
        let meth = match method{
            methods::GET => HttpMethod::Get,
            methods::HEAD => HttpMethod::Head,
            methods::POST => HttpMethod::Post,
            methods::PUT => HttpMethod::Put,
            methods::DELETE => HttpMethod::Delete,
            methods::CONNECT => HttpMethod::Connect,
            methods::OPTIONS => HttpMethod::Options,
            methods::TRACE => HttpMethod::Trace,
            _ => HttpMethod::Unknown(None),
        };
        (*req).set_method(meth);
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_req_set_path(req: *mut DynHttpRequest, path: FfiSlice){
    unsafe{
        let path = path.as_str().to_string();
        (*req).set_path(path);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http_req_write(fut: *mut FfiFuture, req: *mut DynHttpRequest, buf: FfiSlice){
    unsafe{
        let mut req = Box::from_raw(req);
        let fut = Box::from_raw(fut);
        RT.get().unwrap().spawn(async move{
            match req.write(buf.as_bytes()).await{
                Ok(_) => fut.complete(ptr::null_mut()),
                Err(e) => fut.cancel_with_err(IO_ERROR, e.to_string().into()),
            }

            let _ = Box::into_raw(req);
            let _ = Box::into_raw(fut);
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_req_send(fut: *mut FfiFuture, req: *mut DynHttpRequest, buf: FfiSlice){
    unsafe{
        let mut req = Box::from_raw(req);
        let fut = Box::from_raw(fut);

        RT.get().unwrap().spawn(async move{
            match req.send(buf.as_bytes()).await{
                Ok(_) => fut.complete(ptr::null_mut()),
                Err(e) => fut.cancel_with_err(IO_ERROR, e.to_string().into()),
            }

            let _ = Box::into_raw(req);
            let _ = Box::into_raw(fut);
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_req_flush(fut: *mut FfiFuture, req: *mut DynHttpRequest){
    unsafe{
        let fut = Box::from_raw(fut);
        let mut req = Box::from_raw(req);
        RT.get().unwrap().spawn(async move{
            match req.flush().await {
                Ok(_) => fut.complete(ptr::null_mut()),
                Err(e) => fut.cancel_with_err(IO_ERROR, e.to_string().into()),
            }

            let _ = Box::into_raw(fut);
            let _ = Box::into_raw(req);
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http_req_read(fut: *mut FfiFuture, req: *mut DynHttpRequest){
    unsafe{
        let mut req = Box::from_raw(req);
        let fut = Box::from_raw(fut);

        RT.get().unwrap().spawn(async move{
            match req.read_response().await{
                Ok(_) => fut.complete(ptr::null_mut()),
                Err(e) => fut.cancel_with_err(ERROR, e.to_string().into()),
            }

            let _ = Box::into_raw(req);
            let _ = Box::into_raw(fut);
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_req_read_until_complete(fut: *mut FfiFuture, req: *mut DynHttpRequest){
    unsafe{
        let mut req = Box::from_raw(req);
        let fut = Box::from_raw(fut);

        RT.get().unwrap().spawn(async move{
            match req.read_until_complete().await{
                Ok(_) => fut.complete(ptr::null_mut()),
                Err(e) => fut.cancel_with_err(ERROR, e.to_string().into()),
            }

            let _ = Box::into_raw(req);
            let _ = Box::into_raw(fut);
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_req_read_until_head_complete(fut: *mut FfiFuture, req: *mut DynHttpRequest){
    unsafe{
        let mut req = Box::from_raw(req);
        let fut = Box::from_raw(fut);

        RT.get().unwrap().spawn(async move{
            match req.read_until_head_complete().await{
                Ok(_) => fut.complete(ptr::null_mut()),
                Err(e) => fut.cancel_with_err(ERROR, e.to_string().into()),
            }

            let _ = Box::into_raw(req);
            let _ = Box::into_raw(fut);
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http_response_get_status_code(req: *mut DynHttpRequest) -> u16 {
    unsafe {
        (*req).get_response().code
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_response_get_status_msg(req: *mut DynHttpRequest) -> FfiSlice {
    unsafe {
        (&(*req).get_response().status).into()
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_response_has_header(req: *mut DynHttpRequest, name: FfiSlice) -> bool {
    unsafe{
        (*req).get_response().headers.contains_key(name.as_str().as_ref())
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_response_has_header_count(req: *mut DynHttpRequest, name: FfiSlice) -> usize {
    unsafe{
        (*req).get_response().headers.get(name.as_str().as_ref()).and_then(|h|Some(h.len())).unwrap_or(0)
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_response_get_first_header(req: *mut DynHttpRequest, name: FfiSlice) -> FfiSlice {
    unsafe{
        (*req).get_response().headers.get(name.as_str().as_ref()).and_then(|h|Some(FfiSlice::from_string(h[0].clone()))).unwrap_or(FfiSlice::empty())
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_response_get_header(req: *mut DynHttpRequest, name: FfiSlice, index: usize) -> FfiSlice {
    unsafe{
        (*req).get_response().headers.get(name.as_str().as_ref()).and_then(
            |h|h.get(index)
            .and_then(|h|Some(FfiSlice::from_string(h.clone())))
        ).unwrap_or(FfiSlice::empty())
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_response_get_body(req: *mut DynHttpRequest) -> FfiSlice {
    unsafe {
        (&(*req).get_response().body).into()
    }
}
