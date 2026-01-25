use std::fmt::Display;

use tokio::io::{AsyncRead, AsyncWrite};



pub trait Stream: AsyncRead + AsyncWrite + Unpin + Send + Sync {}
impl<A> Stream for A where A: AsyncRead + AsyncWrite + Unpin + Send + Sync {}

#[derive(Debug, Clone)]
pub enum HttpType{
    Http1,
    Http2,
    Http3,
}
impl Display for HttpType{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self{
            Self::Http1 => write!(f, "Http1"),
            Self::Http2 => write!(f, "Http2"),
            Self::Http3 => write!(f, "Http3"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum HttpVersion{
    Unknown(Option<String>),
    Debug,

    Http09,
    Http10,
    Http11,
    Http2,
    Http3,
}
impl Display for HttpVersion{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unknown(s) => if let Some(v) = s { write!(f, "Unknown({})", v) } else { write!(f, "Unknown") },
            Self::Debug => write!(f, "Debug"),
            
            Self::Http09 => write!(f, "HTTP/0.9"),
            Self::Http10 => write!(f, "HTTP/1.0"),
            Self::Http11 => write!(f, "HTTP/1.1"),
            Self::Http2 => write!(f, "HTTP/2"),
            Self::Http3 => write!(f, "HTTP/3"),
        }
    }
}

pub mod server{
    use super::*;

    pub trait HttpClient{}
    pub trait HttpSocket{
        fn get_type() -> HttpType;
    }
}

