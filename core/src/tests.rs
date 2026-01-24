#![allow(unused_imports)]
use std::sync::atomic::Ordering;
#[cfg(test)]

use std::{sync::Arc, thread, time::Duration, ptr};
use crate::ffi::{httpcpp::{add, add_f64, mainthing}, futures::{self, FfiFuture}};

#[test]
fn one_is_one(){
    assert!(1 == 1);
}

#[test]
fn httpcpp_test(){
    unsafe{
        assert!(add_f64(1.0, 2.0) == 3.0);
        assert!(add(1, 2) == 3);
        assert!(mainthing() == 0);
    }
}

#[test]
fn ffi_future_sleep(){
    let fut = Arc::new(FfiFuture::new(None));
    let tfut = fut.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(500));
        tfut.complete(ptr::null_mut());
    });

    assert!(fut.state.load(Ordering::Acquire) == futures::PENDING);
    thread::sleep(Duration::from_millis(500));
    assert!(fut.state.load(Ordering::Acquire) == futures::READY);
    unsafe { assert!(*fut.result.get() == ptr::null_mut()) };

}