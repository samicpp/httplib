use std::fmt::Display;

use tokio::io::{AsyncRead, AsyncWrite};



pub trait ReadStream: AsyncRead + Unpin + Send + Sync {}
impl<A> ReadStream for A where A: AsyncRead + Unpin + Send + Sync {}

pub trait WriteStream: AsyncWrite + Unpin + Send + Sync {}
impl<A> WriteStream for A where A: AsyncWrite + Unpin + Send + Sync {}

pub trait Stream: ReadStream + WriteStream {}
impl<A> Stream for A where A: ReadStream + WriteStream {}


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

#[derive(Debug, Clone)]
pub enum HttpMethod{
    Unknown(Option<String>),

    Get,
    Head,
    Post,
    Put,
    Delete,
    Connect,
    Options,
    Trace,
}
impl Display for HttpMethod{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unknown(s) => if let Some(v) = s { write!(f, "Unknown({})", v) } else { write!(f, "Unknown") },

            Self::Get => write!(f, "GET"),
            Self::Head => write!(f, "HEAD"),
            Self::Post => write!(f, "POST"),
            Self::Put => write!(f, "PUT"),
            Self::Delete => write!(f, "DELETE"),
            Self::Connect => write!(f, "CONNECT"),
            Self::Options => write!(f, "OPTIONS"),
            Self::Trace => write!(f, "TRACE"),
        }
    }
}
impl HttpMethod{
    pub fn from(string: &str) -> Self{
        if string.eq_ignore_ascii_case("get") { Self::Get }
        else if string.eq_ignore_ascii_case("head") { Self::Head }
        else if string.eq_ignore_ascii_case("post") { Self::Post }
        else if string.eq_ignore_ascii_case("put") { Self::Put }
        else if string.eq_ignore_ascii_case("delete") { Self::Delete }
        else if string.eq_ignore_ascii_case("connect") { Self::Connect }
        else if string.eq_ignore_ascii_case("options") { Self::Options }
        else if string.eq_ignore_ascii_case("trace") { Self::Trace }
        else { Self::Unknown(Some(string.to_owned())) }
    }
}


pub mod server{
    use std::{collections::HashMap, pin::Pin};

    use super::*;

    pub trait HttpClient{
        fn get_method<'_a>(&'_a self) -> &'_a HttpMethod;
        fn get_path<'_a>(&'_a self) -> &'_a str;
        fn get_version<'_a>(&'_a self) -> &'_a HttpVersion;

        fn get_headers<'_a>(&'_a self) -> &'_a HashMap<String, Vec<String>>;
        fn get_body<'_a>(&'_a self) -> &'_a [u8];
    }
    pub trait HttpSocket{
        fn get_type() -> HttpType;

        fn get_client<'_a>(&'_a self) -> &'_a dyn HttpClient;
        fn read_client<'_a>(&'_a mut self) -> Pin<Box<dyn Future<Output = Result<&'_a dyn HttpClient, std::io::Error>> + '_a>>;

        fn add_header(&mut self, header: &str, value: &str);
        fn set_header(&mut self, header: &str, value: &str);
        fn del_header(&mut self, header: &str) -> Option<Vec<String>>;
        
        fn set_status(&mut self, code: u16, message: String);
        fn close<'a>(&'a mut self, body: &'a [u8] ) -> Pin<Box<dyn Future<Output = Result<(), std::io::Error>> + Send + 'a>>;
        fn write<'a>(&'a mut self, body: &'a [u8] ) -> Pin<Box<dyn Future<Output = Result<(), std::io::Error>> + Send + 'a>>;
    }

    pub type DynHttpSocket = Box<dyn HttpSocket>;
}

