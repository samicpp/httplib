#![allow(dead_code)]
#![allow(unused_imports)]

use std::net::{SocketAddr, ToSocketAddrs};
use http::{extra::PolyHttpSocket, http1::{client::Http1Request, server::Http1Socket}, shared::{HttpRequest, HttpSocket, HttpVersion, ReadStream, WriteStream}};
use crate::{clients::{tcp_connect, tls_upgrade}, httpcpp::{add, add_f64, add_test, server_test}, servers::{Server, TcpServer, tcp_serve}};

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
    let mut http = Http1Socket::new(sock.to_stream(), 8 * 1024);
    // let mut http = PolyHttpSocket::Http1(http);
    
    println!("{}", addr);
    let mut client = http.read_until_complete().await.unwrap().clone();
    let body = client.body;
    client.body = Vec::with_capacity(0);
    dbg!(&client);
    println!("body = {}", String::from_utf8_lossy(&body));

    if client.valid{
        assert_eq!(client.head_complete, true);
        assert_eq!(client.body_complete, true);
    }
    
    if !client.valid {
        http.set_status(400, "Bad Request".to_owned());
        http.close(b"fix your client").await.unwrap();
    }
    else if client.version.is_unknown() {
        http.version_override = Some(HttpVersion::Http11);
        http.set_status(400, "Bad Request".to_owned());
        http.close(format!("\"{}\" is not a valid version", if let HttpVersion::Unknown(Some(u)) = client.version { u.clone() } else { "???".to_owned() }).as_bytes()).await.unwrap();
    }
    else if client.method.is_unknown() {
        http.set_status(405, "Method Not Allowed".to_owned());
        http.set_header("Allow", "GET, HEAD, POST, PUT, DELETE, CONNECT, OPTIONS, TRACE");
        http.close(b"erm, what are you trying to do?").await.unwrap();
    }
    else if client.version.is_http11() && client.host.is_none(){
        http.set_status(400, "Bad Request".to_owned());
        http.close(b"what're you connecting to, what host?").await.unwrap();
    }
    else {
        http.set_status(200, "OK".to_owned());
        http.set_header("Content-Type", "text/plain");
        http.close(b"everything's alright").await.unwrap();
    }
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
    let tcp = tcp_connect("google.com:443").await.unwrap();
    let tls = tls_upgrade(tcp, "www.google.com".to_owned(), vec![b"http/1.1".to_vec()]).await.unwrap();
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