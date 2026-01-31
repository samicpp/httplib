use http::shared::Stream;
use tokio::net::TcpStream;
use tokio_rustls::TlsStream;

pub mod tests;
pub mod servers;
pub mod ffi;
pub mod httpcpp;
pub mod errno;
pub mod clients;


pub enum DynStream {
    Tcp(TcpStream),
    TcpTls(TlsStream<TcpStream>),
}
impl DynStream{
    pub fn to_stream(self) -> Box<dyn Stream>{
        match self{
            Self::Tcp(tcp) => Box::new(tcp),
            Self::TcpTls(tls) => Box::new(tls),
        }
    }
}
impl From<TcpStream> for DynStream{
    fn from(value: TcpStream) -> Self {
        Self::Tcp(value)
    }
}
impl From<TlsStream<TcpStream>> for DynStream{
    fn from(value: TlsStream<TcpStream>) -> Self {
        Self::TcpTls(value)
    }
}
impl From<tokio_rustls::client::TlsStream<TcpStream>> for DynStream{
    fn from(value: tokio_rustls::client::TlsStream<TcpStream>) -> Self {
        Self::TcpTls(TlsStream::Client(value))
    }
}
impl From<tokio_rustls::server::TlsStream<TcpStream>> for DynStream{
    fn from(value: tokio_rustls::server::TlsStream<TcpStream>) -> Self {
        Self::TcpTls(TlsStream::Server(value))
    }
}

