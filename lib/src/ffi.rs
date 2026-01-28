use std::{ffi::{CStr, c_void}, net::SocketAddr, ptr};

use http::shared::{HttpClient, HttpMethod, HttpSocket, HttpVersion};
use httprs_core::ffi::{futures::FfiFuture, own::{FfiSlice, RT}};

use crate::servers::{DynHttpSocket, Server, TcpServer};


pub struct FfiBundle{
    pub http: DynHttpSocket,
    pub addr: SocketAddr,
}
pub struct FfiServer{
    pub boxed: Box<dyn Server + Send>,
}

#[repr(C)]
#[derive(Debug)]
pub struct FfiClient{
    pub owned: bool,
    pub valid: bool,

    pub head_complete: bool,
    pub body_complete: bool,
    
    pub path: FfiSlice,
    pub method: u8,
    pub version: u8,
    pub method_str: FfiSlice,

    pub headers_len: usize,
    pub headers_cap: usize,
    pub headers: *const FfiHeaderPair,
    pub body: FfiSlice,

    pub host: FfiSlice,
    pub scheme: FfiSlice,
}

#[repr(C)]
#[derive(Debug)]
pub struct FfiHeaderPair{
    pub nam: FfiSlice,
    pub val: FfiSlice,
}

impl FfiClient{
    pub fn from_owned(client: HttpClient) -> Self{
        let mut pairs = Vec::new();
        client.headers.into_iter().for_each(|(h,vs)|vs.into_iter().for_each(|v| pairs.push(FfiHeaderPair { nam: FfiSlice::from_string(h.clone()), val: FfiSlice::from_string(v) })));
        let pair_ptr = pairs.as_ptr();
        let pairs_len = pairs.len();
        let pairs_cap = pairs.capacity();
        std::mem::forget(pairs);

        Self {
            owned: true,
            valid: client.valid,

            head_complete: client.head_complete,
            body_complete: client.body_complete,

            path: FfiSlice::from_string(client.path),
            method: match client.method { HttpMethod::Unknown(_) => 0, HttpMethod::Get => 1, HttpMethod::Head => 2, HttpMethod::Post => 3, HttpMethod::Put => 4, HttpMethod::Delete => 5, HttpMethod::Connect => 6, HttpMethod::Options => 7, HttpMethod::Trace => 8 },
            version: match client.version { HttpVersion::Unknown(_) => 0, HttpVersion::Debug => 1, HttpVersion::Http09 => 2, HttpVersion::Http10 => 3, HttpVersion::Http11 => 4, HttpVersion::Http2 => 5, HttpVersion::Http3 => 6 },
            method_str: FfiSlice::from_string(client.method.to_string()),

            headers_len: pairs_len,
            headers_cap: pairs_cap,
            headers: pair_ptr,
            body: FfiSlice::from_vec(client.body),

            host: client.host.and_then(|h|Some(FfiSlice::from_string(h))).unwrap_or(FfiSlice::empty()),
            scheme: client.scheme.and_then(|s|Some(FfiSlice::from_string(s))).unwrap_or(FfiSlice::empty()),
        }
    }
    pub fn from(client: &HttpClient) -> Self{
        let mut pairs = Vec::new();
        client.headers.iter().for_each(|(h,vs)|vs.into_iter().for_each(|v| pairs.push(FfiHeaderPair { nam: FfiSlice::from_str(h), val: FfiSlice::from_str(v) })));
        let pair_ptr = pairs.as_ptr();
        let pairs_len = pairs.len();
        let pairs_cap = pairs.capacity();
        std::mem::forget(pairs);

        Self {
            owned: false,
            valid: client.valid,

            head_complete: client.head_complete,
            body_complete: client.body_complete,

            path: FfiSlice::from_str(&client.path),
            method: match client.method { HttpMethod::Unknown(_) => 0, HttpMethod::Get => 1, HttpMethod::Head => 2, HttpMethod::Post => 3, HttpMethod::Put => 4, HttpMethod::Delete => 5, HttpMethod::Connect => 6, HttpMethod::Options => 7, HttpMethod::Trace => 8 },
            version: match client.version { HttpVersion::Unknown(_) => 0, HttpVersion::Debug => 1, HttpVersion::Http09 => 2, HttpVersion::Http10 => 3, HttpVersion::Http11 => 4, HttpVersion::Http2 => 5, HttpVersion::Http3 => 6 },
            method_str: FfiSlice::from_string(client.method.to_string()),

            headers_len: pairs_len,
            headers_cap: pairs_cap,
            headers: pair_ptr,
            body: FfiSlice::from_buf(&client.body),

            host: client.host.as_ref().and_then(|h|Some(FfiSlice::from_str(h))).unwrap_or(FfiSlice::empty()),
            scheme: client.scheme.as_ref().and_then(|s|Some(FfiSlice::from_str(s))).unwrap_or(FfiSlice::empty()),
        }
    }

    pub fn free(self){
        self.method_str.free();
        let pairs = unsafe { Vec::from_raw_parts(self.headers as *mut FfiHeaderPair, self.headers_len, self.headers_cap) };
        
        if self.owned{
            self.path.free();
            self.body.free();
            self.host.free();
            self.scheme.free();


            for h in pairs {
                h.nam.free();
                h.val.free();
            }
        }
    }
}


#[unsafe(no_mangle)]
pub extern "C" fn server_new_tcp(fut: *mut FfiFuture, string: *mut i8){
    unsafe {
        let addr = CStr::from_ptr(string).to_string_lossy().to_string();
        let fut = Box::from_raw(fut);

        RT.get().unwrap().spawn(async move {
            match TcpServer::new(addr).await{
                Ok(r) => {
                    let boxed: Box<dyn Server + Send> = Box::new(r);
                    let wrap = Box::new(FfiServer { boxed });
                    let ptr = Box::into_raw(wrap);
                    fut.complete(ptr as *mut c_void)
                },
                Err(_) => fut.cancel(),
            }
            let _ = Box::into_raw(fut);
        });
    }
}

// #[allow(improper_ctypes_definitions)]
#[unsafe(no_mangle)]
pub extern "C" fn server_accept(fut: *mut FfiFuture, server: *mut FfiServer){
    unsafe {
        let mut server = Box::from_raw(server);
        let fut = Box::from_raw(fut);

        RT.get().unwrap().spawn(async move {
            match server.boxed.accept().await{
                Ok((addr, http)) => {
                    // let boxed = Box::new(http);
                    // let ptr = Box::into_raw(boxed);

                    let ffi = FfiBundle {
                        http,
                        addr,
                    };

                    fut.complete(Box::into_raw(Box::new(ffi)) as *mut c_void)
                },
                Err(_) => fut.cancel(),
            }

            let _ = Box::into_raw(server);
            let _ = Box::into_raw(fut);
        });
    }
}
// #[allow(improper_ctypes_definitions)]
#[unsafe(no_mangle)]
pub extern "C" fn server_loop(fut: *mut FfiFuture, server: *mut FfiServer, cb: extern "C" fn(*mut FfiBundle)){
    unsafe {
        let mut ser = Box::from_raw(server);
        let fut = Box::from_raw(fut);

        RT.get().unwrap().spawn(async move {
            loop {
                match ser.boxed.accept().await{
                    Ok((addr, http)) => cb(Box::into_raw(Box::new(FfiBundle { http, addr }))),
                    Err(_) => {
                        fut.cancel();
                        break;
                    },
                }
            }

            let _ = Box::into_raw(ser);
            let _ = Box::into_raw(fut);
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn get_addr_str(ffi: *mut FfiBundle) -> FfiSlice{
    unsafe{
        FfiSlice::from_string((*ffi).addr.to_string())
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http_read_client(fut: *mut FfiFuture, ffi: *mut FfiBundle){
    unsafe{
        let mut ffi = Box::from_raw(ffi);
        let fut = Box::from_raw(fut);

        RT.get().unwrap().spawn(async move{
            match ffi.http.read_client().await{
                Ok(_) => fut.complete(ptr::null_mut()),
                Err(_) => fut.cancel(),
            }

            let _ = Box::into_raw(ffi);
            let _ = Box::into_raw(fut);
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_read_until_complete(fut: *mut FfiFuture, ffi: *mut FfiBundle){
    unsafe{
        let mut ffi = Box::from_raw(ffi);
        let fut = Box::from_raw(fut);

        RT.get().unwrap().spawn(async move{
            match ffi.http.read_until_complete().await{
                Ok(_) => fut.complete(ptr::null_mut()),
                Err(_) => fut.cancel(),
            }

            let _ = Box::into_raw(ffi);
            let _ = Box::into_raw(fut);
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_read_until_head_complete(fut: *mut FfiFuture, ffi: *mut FfiBundle){
    unsafe{
        let mut ffi = Box::from_raw(ffi);
        let fut = Box::from_raw(fut);

        RT.get().unwrap().spawn(async move{
            match ffi.http.read_until_head_complete().await{
                Ok(_) => fut.complete(ptr::null_mut()),
                Err(_) => fut.cancel(),
            }

            let _ = Box::into_raw(ffi);
            let _ = Box::into_raw(fut);
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http_set_header(ffi: *mut FfiBundle, pair: FfiHeaderPair){
    unsafe{
        let name = pair.nam.as_str();
        let value = pair.val.as_str();

        (*ffi).http.set_header(&name, &value);
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_add_header(ffi: *mut FfiBundle, pair: FfiHeaderPair){
    unsafe{
        let name = pair.nam.as_str();
        let value = pair.val.as_str();

        (*ffi).http.add_header(&name, &value);
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_del_header(ffi: *mut FfiBundle, name: FfiSlice){
    unsafe{
        let name = name.as_str();
        let _ = (*ffi).http.del_header(&name);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http_write(fut: *mut FfiFuture, ffi: *mut FfiBundle, buf: FfiSlice){
    unsafe{
        let mut ffi = Box::from_raw(ffi);
        let fut = Box::from_raw(fut);
        RT.get().unwrap().spawn(async move{
            match ffi.http.write(buf.as_bytes()).await{
                Ok(_) => fut.complete(ptr::null_mut()),
                Err(_) => fut.cancel(),
            }

            let _ = Box::into_raw(ffi);
            let _ = Box::into_raw(fut);
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_close(fut: *mut FfiFuture, ffi: *mut FfiBundle, buf: FfiSlice){
    unsafe{
        let mut ffi = Box::from_raw(ffi);
        let fut = Box::from_raw(fut);

        RT.get().unwrap().spawn(async move{
            match ffi.http.close(buf.as_bytes()).await{
                Ok(_) => {
                    // println!("normal closure");
                    fut.complete(ptr::null_mut())
                },
                Err(_) => {
                    // dbg!(e);
                    fut.cancel();
                },
            }

            let _ = Box::into_raw(ffi);
            let _ = Box::into_raw(fut);
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http_get_fficlient(ffi: *mut FfiBundle) -> *mut FfiClient {
    unsafe{
        Box::into_raw(Box::new(FfiClient::from(&(*ffi).http.get_client())))
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http_free_fficlient(ffi: *mut FfiClient) {
    unsafe { 
        let cl = Box::from_raw(ffi);
        cl.free();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http_client_has_header(ffi: *mut FfiBundle, name: FfiSlice) -> bool {
    unsafe{
        (*ffi).http.get_client().headers.contains_key(name.as_str().as_ref())
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http_client_has_header_count(ffi: *mut FfiBundle, name: FfiSlice) -> usize {
    unsafe{
        (*ffi).http.get_client().headers.get(name.as_str().as_ref()).and_then(|h|Some(h.len())).unwrap_or(0)
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http_client_get_first_header(ffi: *mut FfiBundle, name: FfiSlice) -> FfiSlice {
    unsafe{
        (*ffi).http.get_client().headers.get(name.as_str().as_ref()).and_then(|h|Some(FfiSlice::from_string(h[0].clone()))).unwrap_or(FfiSlice::empty())
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http_client_get_header(ffi: *mut FfiBundle, name: FfiSlice, index: usize) -> FfiSlice {
    unsafe{
        (*ffi).http.get_client().headers.get(name.as_str().as_ref()).and_then(
            |h|h.get(index)
            .and_then(|h|Some(FfiSlice::from_string(h.clone())))
        ).unwrap_or(FfiSlice::empty())
    }
}
