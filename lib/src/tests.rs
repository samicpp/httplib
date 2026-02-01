#![allow(dead_code)]
#![allow(unused_imports)]

use std::net::{SocketAddr, ToSocketAddrs};
use http::{extra::PolyHttpSocket, http1::{client::Http1Request, server::Http1Socket}, shared::{HttpSocket, HttpRequest, ReadStream, WriteStream}};
use crate::{clients::tls_connect, httpcpp::{add, add_f64, add_test, server_test}, servers::{Server, TcpServer, tcp_serve}};

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
    let (addr, sock) = tcp.accept().await.unwrap();
    let http = PolyHttpSocket::Http1(Http1Socket::new(sock.to_stream(), 8 * 1024));
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

#[ignore = "user interaction"]
#[test]
fn test_over_ffi(){
    std::thread::spawn(move || {
        unsafe {
            assert_eq!(server_test(), 0);
        }
    }).join().unwrap();
}

#[ignore = "uses network"]
#[tokio::test]
async fn request_google(){
    let tls = tls_connect("google.com:443", "www.google.com".to_owned()).await.unwrap();
    let mut req = Http1Request::new(Box::new(tls), 8 * 1024);
    req.set_path("/".to_owned());
    req.set_header("Host", "www.google.com");
    req.send(b"").await.unwrap();
    let _ = req.read_until_complete().await.unwrap();
    let body = req.response.body;
    req.response.body = Vec::new();
    
    println!("body.len() == {}", body.len());
    dbg!(&req);
    dbg!(&req.response);
    println!("{}", String::from_utf8_lossy(&body));
}