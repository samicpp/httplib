#![allow(dead_code)]
#![allow(unused_imports)]

use std::net::SocketAddr;
use http::{extra::PolyHttpSocket, shared::{ReadStream, WriteStream, HttpSocket}};
use crate::{httpcpp::{add, add_f64, add_test, server_test}, servers::{Server, TcpServer, tcp_serve}};

#[cfg(test)]


#[test]
fn four_is_four(){
    assert!(4 == 4);
}

#[ignore = "requires user input"]
#[tokio::test]
async fn serve_tcp(){
    // tcp_serve("0.0.0.0:1024".to_owned(), |a,h| handler(a,h)).await.unwrap();
    let mut tcp = TcpServer::new("0.0.0.0:1024".to_owned()).await.unwrap();
    let (addr, http) = tcp.accept().await.unwrap();
    handler(addr, http).await;

}

async fn handler<R: ReadStream, W: WriteStream>(_addr: SocketAddr, mut http: PolyHttpSocket<R, W>){
    http.read_until_head_complete().await.unwrap();
    http.close(b"body").await.unwrap();
}

#[test]
fn httpcpp_test(){
    unsafe{
        assert_eq!(add_f64(1.0, 2.0), 3.0);
        assert_eq!(add(1, 2), 3);
        assert_eq!(add_test(), 0);
    }
}

#[ignore = "nonfunctional"]
#[test]
fn test_over_ffi(){
    std::thread::spawn(move || {
        unsafe {
            assert_eq!(server_test(), 0);
        }
    }).join().unwrap();
}
