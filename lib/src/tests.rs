#![allow(dead_code)]
#![allow(unused_imports)]

use std::net::SocketAddr;
use http::{extra::PolyHttpSocket, shared::{ReadStream, WriteStream, server::HttpSocket}};
use crate::servers::{Server, TcpServer, tcp_serve};

#[cfg(test)]


#[test]
fn four_is_four(){
    assert!(4 == 4);
}

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
