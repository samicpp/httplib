use std::{net::SocketAddr, pin::Pin};

use http::{extra::PolyHttpSocket, http1::server::Http1Socket, http2::PREFACE, shared::Stream};
use tokio::{io::{ReadHalf, WriteHalf}, net::{TcpListener, TcpStream}};

use crate::DynStream;


pub async fn tcp_serve<F, Fut, O>(address: String, handler: F) -> std::io::Result<()>
where 
    F: Fn(SocketAddr, PolyHttpSocket<ReadHalf<TcpStream>, WriteHalf<TcpStream>>) -> Fut + Send + Clone + Copy + Sync + 'static,
    Fut: Future<Output = O> + Send + 'static,
{
    let listener = TcpListener::bind(address).await?;

    loop{
        let (socket, adddress) = listener.accept().await?;
        tokio::spawn(async move{
            let http = Http1Socket::new(socket, 8 * 1024);
            // let http: Box<dyn HttpSocket + Send> = Box::new(http);
            let http = PolyHttpSocket::Http1(http);
            handler(adddress, http).await;
        });
    }
}

pub type DynHttpSocket = PolyHttpSocket<ReadHalf<Box<dyn Stream>>, WriteHalf<Box<dyn Stream>>>;

pub trait Server{
    fn accept<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = Result<(SocketAddr, DynStream), std::io::Error>> + Send + 'a>>;
}

pub struct TcpServer{
    // cb: Arc<dyn Fn(SocketAddr, PolyHttpSocket<ReadHalf<TcpStream>, WriteHalf<TcpStream>>) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>> + Send + Sync + 'static>,
    listener: TcpListener,
}
impl Server for TcpServer{
    fn accept<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = Result<(SocketAddr, DynStream), std::io::Error>> + Send + 'a>> {
        Box::pin(async move{
            let (s, addr) = self.listener.accept().await?;
            let sock = DynStream::Tcp(s);
            // let http = Http1Socket::new(sock, 8 * 1024);
            // let http = PolyHttpSocket::Http1(http);
            Ok((addr, sock))
        })
    }
}
impl TcpServer{
    pub async fn new(address: String) -> std::io::Result<Self>{
        Ok(Self {
            listener: TcpListener::bind(address).await?,
        })
    }
}

pub async fn detect_prot(tcp: &mut TcpStream) -> u8{
    let mut peek = [0u8; 24];
    let _ = tcp.peek(&mut peek).await;

    if peek == PREFACE{
        1 // http2
    }
    else if peek[0] == 22{
        2 // tls
    }
    else {
        0 // idk
    }
}