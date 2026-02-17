use core::slice;
use std::{ptr, sync::Arc};

use http::{http2::{client::Http2Request, core::{Http2Frame, Http2Settings}, server::Http2Socket, session::{Http2Session, Mode}}, shared::LibError};
use httprs_core::ffi::{futures::FfiFuture, own::{FfiSlice, RT}};
use tokio::io::{BufReader, ReadHalf, WriteHalf};

use crate::{DynStream, clients::DynHttpRequest, errno::{ERROR, IO_ERROR}, ffi::{server::FfiHeaderPair, utils::{heap_ptr, heap_void_ptr}}, servers::DynHttpSocket};

pub type DynH2Sess = Http2Session<BufReader<ReadHalf<DynStream>>, WriteHalf<DynStream>>;



#[unsafe(no_mangle)]
pub extern "C" fn http2_new(stream: *mut DynStream, bufsize: usize) -> *const DynH2Sess {
    unsafe {
        let stream = *Box::from_raw(stream);
        let (netr, netw) = tokio::io::split(stream);
        let netr = BufReader::with_capacity(bufsize, netr);
        let h2 = Http2Session::with(netr, netw, Mode::Ambiguous, true, Http2Settings::default());
        let h2 = Arc::into_raw(Arc::new(h2));
        h2
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http2_new_client(stream: *mut DynStream, bufsize: usize) -> *const DynH2Sess {
    unsafe {
        let stream = *Box::from_raw(stream);
        let (netr, netw) = tokio::io::split(stream);
        let netr = BufReader::with_capacity(bufsize, netr);
        let h2 = Http2Session::with(netr, netw, Mode::Client, true, Http2Settings::default());
        let h2 = Arc::into_raw(Arc::new(h2));
        h2
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http2_new_server(stream: *mut DynStream, bufsize: usize) -> *const DynH2Sess {
    unsafe {
        let stream = *Box::from_raw(stream);
        let (netr, netw) = tokio::io::split(stream);
        let netr = BufReader::with_capacity(bufsize, netr);
        let h2 = Http2Session::with(netr, netw, Mode::Server, true, Http2Settings::default());
        let h2 = Arc::into_raw(Arc::new(h2));
        h2
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http2_with(stream: *mut DynStream, bufsize: usize, mode: u8, strict: bool, settings: FfiSlice) -> *const DynH2Sess {
    unsafe {
        let stream = *Box::from_raw(stream);
        let (netr, netw) = tokio::io::split(stream);
        let netr = BufReader::with_capacity(bufsize, netr);

        let mode = match mode {
            1 => Mode::Client,
            2 => Mode::Server,
            _ => Mode::Ambiguous,
        };

        let settings = Http2Settings::from(settings.as_bytes());

        let h2 = Http2Session::with(netr, netw, mode, strict, settings);
        let h2 = Arc::into_raw(Arc::new(h2));
        h2
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http2_free(session: *const DynH2Sess) {
    unsafe {
        drop(Arc::from_raw(session));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http2_read_preface(fut: *const FfiFuture, session: *const DynH2Sess) {
    unsafe {
        let sess = &*session;
        let fut = &*fut;

        RT.get().unwrap().spawn(async move{
            match sess.read_preface().await {
                Ok(success) => fut.complete(heap_void_ptr(success)),
                Err(e) => fut.cancel_with_err(IO_ERROR, e.to_string().into()),
            }
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http2_send_preface(fut: *const FfiFuture, session: *const DynH2Sess) {
    unsafe {
        let sess = &*session;
        let fut = &*fut;

        RT.get().unwrap().spawn(async move{
            match sess.send_preface().await {
                Ok(_) => fut.complete(ptr::null_mut()),
                Err(e) => fut.cancel_with_err(IO_ERROR, e.to_string().into()),
            }
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http2_next(fut: *const FfiFuture, session: *const DynH2Sess) {
    unsafe {
        let sess = &*session;
        let fut = &*fut;

        RT.get().unwrap().spawn(async move{
            match sess.next().await {
                Ok(None) => fut.complete(ptr::null_mut()),
                Ok(Some(opened)) => fut.complete(heap_void_ptr(opened)),
                Err(LibError::Io(io)) => fut.cancel_with_err(IO_ERROR, io.to_string().into()),
                Err(e) => fut.cancel_with_err(ERROR, e.to_string().into()),
            }
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http2_read_raw(fut: *const FfiFuture, session: *const DynH2Sess) {
    unsafe {
        let sess = &*session;
        let fut = &*fut;

        RT.get().unwrap().spawn(async move{
            match sess.read_frame().await {
                Ok(frame) => fut.complete(heap_void_ptr(FfiSlice::from_vec(frame.source))),
                Err(e) => fut.cancel_with_err(IO_ERROR, e.to_string().into()),
            }
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http2_handle_raw(fut: *const FfiFuture, session: *const DynH2Sess, frame: FfiSlice) {
    unsafe {
        let sess = &*session;
        let fut = &*fut;
        let frame = Http2Frame::from_owned(if frame.owned { frame.to_vec().unwrap() } else { frame.as_bytes().to_vec() });
        let frame = if let Some(frame) = frame { frame } else { return };

        RT.get().unwrap().spawn(async move{
            match sess.handle(frame).await {
                Ok(None) => fut.complete(ptr::null_mut()),
                Ok(Some(opened)) => fut.complete(heap_void_ptr(opened)),
                Err(LibError::Io(io)) => fut.cancel_with_err(IO_ERROR, io.to_string().into()),
                Err(e) => fut.cancel_with_err(ERROR, e.to_string().into()),
            }
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http2_open_stream(session: *const DynH2Sess) -> u32 {
    unsafe {
        (*session).open_stream().unwrap_or(0)
    }
}


#[unsafe(no_mangle)]
pub extern "C" fn http2_send_data(fut: *const FfiFuture, session: *const DynH2Sess, stream_id: u32, end: bool, buf: FfiSlice) {
    unsafe {
        let sess = &*session;
        let fut = &*fut;

        RT.get().unwrap().spawn(async move{
            match sess.send_data(stream_id, end, buf.as_bytes()).await {
                Ok(_) => fut.complete(ptr::null_mut()),
                Err(LibError::Io(io)) => fut.cancel_with_err(IO_ERROR, io.to_string().into()),
                Err(e) => fut.cancel_with_err(ERROR, e.to_string().into()),
            }
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http2_send_headers(fut: *const FfiFuture, session: *const DynH2Sess, stream_id: u32, end: bool, headers: *const FfiHeaderPair, length: usize) {
    unsafe {
        let sess = &*session;
        let fut = &*fut;
        let mut head = Vec::with_capacity(length);

        for hv in slice::from_raw_parts(headers, length) {
            head.push((hv.nam.as_bytes(), hv.val.as_bytes()));
        }

        RT.get().unwrap().spawn(async move{
            match sess.send_headers(stream_id, end, &head).await {
                Ok(_) => fut.complete(ptr::null_mut()),
                Err(LibError::Io(io)) => fut.cancel_with_err(IO_ERROR, io.to_string().into()),
                Err(e) => fut.cancel_with_err(ERROR, e.to_string().into()),
            }
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http2_send_priority(fut: *const FfiFuture, session: *const DynH2Sess, stream_id: u32, dependency: u32, weight: u8) {
    unsafe {
        let sess = &*session;
        let fut = &*fut;

        RT.get().unwrap().spawn(async move{
            match sess.send_priority(stream_id, dependency, weight).await {
                Ok(_) => fut.complete(ptr::null_mut()),
                Err(e) => fut.cancel_with_err(IO_ERROR, e.to_string().into()),
            }
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http2_send_rst_stream(fut: *const FfiFuture, session: *const DynH2Sess, stream_id: u32, code: u32) {
    unsafe {
        let sess = &*session;
        let fut = &*fut;

        RT.get().unwrap().spawn(async move{
            match sess.send_rst_stream(stream_id, code).await {
                Ok(_) => fut.complete(ptr::null_mut()),
                Err(e) => fut.cancel_with_err(IO_ERROR, e.to_string().into()),
            }
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http2_send_settings(fut: *const FfiFuture, session: *const DynH2Sess, settings: FfiSlice) {
    unsafe {
        let sess = &*session;
        let fut = &*fut;
        let settings = Http2Settings::from(settings.as_bytes());

        RT.get().unwrap().spawn(async move{
            match sess.send_settings(settings).await {
                Ok(_) => fut.complete(ptr::null_mut()),
                Err(e) => fut.cancel_with_err(IO_ERROR, e.to_string().into()),
            }
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http2_send_settings_default(fut: *const FfiFuture, session: *const DynH2Sess) {
    unsafe {
        let sess = &*session;
        let fut = &*fut;
        let settings = Http2Settings::default();

        RT.get().unwrap().spawn(async move{
            match sess.send_settings(settings).await {
                Ok(_) => fut.complete(ptr::null_mut()),
                Err(e) => fut.cancel_with_err(IO_ERROR, e.to_string().into()),
            }
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http2_send_settings_default_no_push(fut: *const FfiFuture, session: *const DynH2Sess) {
    unsafe {
        let sess = &*session;
        let fut = &*fut;
        let settings = Http2Settings::default_no_push();

        RT.get().unwrap().spawn(async move{
            match sess.send_settings(settings).await {
                Ok(_) => fut.complete(ptr::null_mut()),
                Err(e) => fut.cancel_with_err(IO_ERROR, e.to_string().into()),
            }
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http2_send_settings_maximum(fut: *const FfiFuture, session: *const DynH2Sess) {
    unsafe {
        let sess = &*session;
        let fut = &*fut;
        let settings = Http2Settings::maximum();

        RT.get().unwrap().spawn(async move{
            match sess.send_settings(settings).await {
                Ok(_) => fut.complete(ptr::null_mut()),
                Err(e) => fut.cancel_with_err(IO_ERROR, e.to_string().into()),
            }
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http2_send_push_promise(fut: *const FfiFuture, session: *const DynH2Sess, associate_id: u32, promise_id: u32, headers: *const FfiHeaderPair, length: usize) {
    unsafe {
        let sess = &*session;
        let fut = &*fut;
        let mut head = Vec::with_capacity(length);

        for hv in slice::from_raw_parts(headers, length) {
            head.push((hv.nam.as_bytes(), hv.val.as_bytes()));
        }

        RT.get().unwrap().spawn(async move{
            match sess.send_push_promise(associate_id, promise_id, &head).await {
                Ok(_) => fut.complete(ptr::null_mut()),
                Err(LibError::Io(io)) => fut.cancel_with_err(IO_ERROR, io.to_string().into()),
                Err(e) => fut.cancel_with_err(ERROR, e.to_string().into()),
            }
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http2_send_ping(fut: *const FfiFuture, session: *const DynH2Sess, ack: bool, buf: FfiSlice) {
    unsafe {
        let sess = &*session;
        let fut = &*fut;

        RT.get().unwrap().spawn(async move{
            match sess.send_ping(ack, buf.as_bytes()).await {
                Ok(_) => fut.complete(ptr::null_mut()),
                Err(e) => fut.cancel_with_err(IO_ERROR, e.to_string().into()),
            }
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http2_send_goaway(fut: *const FfiFuture, session: *const DynH2Sess, stream_id: u32, code: u32, buf: FfiSlice) {
    unsafe {
        let sess = &*session;
        let fut = &*fut;

        RT.get().unwrap().spawn(async move{
            match sess.send_goaway(stream_id, code, buf.as_bytes()).await {
                Ok(_) => fut.complete(ptr::null_mut()),
                Err(e) => fut.cancel_with_err(IO_ERROR, e.to_string().into()),
            }
        });
    }
}


// Arc::increment_strong_count(conf);
// Arc::from_raw(conf)

#[unsafe(no_mangle)]
pub extern "C" fn http2_client_handler(session: *const DynH2Sess, stream_id: u32) -> *mut DynHttpRequest {
    unsafe {
        let session = {
            Arc::increment_strong_count(session);
            Arc::from_raw(session)
        };

        if let Ok(req) = Http2Request::new(stream_id, session) {
            let req = DynHttpRequest::Http2(req);
            heap_ptr(req)
        }
        else {
            ptr::null_mut()
        }
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http2_server_handler(session: *const DynH2Sess, stream_id: u32) -> *mut DynHttpSocket {
    unsafe {
        let session = {
            Arc::increment_strong_count(session);
            Arc::from_raw(session)
        };
        
        if let Ok(req) = Http2Socket::new(stream_id, session) {
            let req = DynHttpSocket::Http2(req);
            heap_ptr(req)
        }
        else {
            ptr::null_mut()
        }
    }
}
