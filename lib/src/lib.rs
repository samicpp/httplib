use std::sync::{Arc, LazyLock};

use http::shared::Stream;
use rustls::crypto::CryptoProvider;
use tokio::net::TcpStream;
use tokio_rustls::TlsStream;

pub mod tests;
pub mod servers;
pub mod ffi;
pub mod httpcpp;
pub mod errno;
pub mod clients;


pub static PROVIDER: LazyLock<Arc<CryptoProvider>> = LazyLock::new(|| Arc::new(rustls::crypto::aws_lc_rs::default_provider()));

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

    pub fn is_tcp(&self) -> bool {
        if let Self::Tcp(_) = self { true }
        else { false }
    }
    pub fn is_tls(&self) -> bool {
        if let Self::TcpTls(_) = self { true }
        else { false }
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

