use std::{fmt::Display, collections::HashMap, pin::Pin};

use tokio::io::{AsyncRead, AsyncWrite};



pub trait ReadStream: AsyncRead + Unpin + Send + Sync {}
impl<A> ReadStream for A where A: AsyncRead + Unpin + Send + Sync {}

pub trait WriteStream: AsyncWrite + Unpin + Send + Sync {}
impl<A> WriteStream for A where A: AsyncWrite + Unpin + Send + Sync {}

pub trait Stream: ReadStream + WriteStream {}
impl<A> Stream for A where A: ReadStream + WriteStream {}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, Eq, PartialEq)]
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



/*pub trait HttpClient{
    fn is_valid(&self) -> bool;
    fn is_complete(&self) -> (bool, bool);
    
    fn get_method<'_a>(&'_a self) -> &'_a HttpMethod;
    fn get_path<'_a>(&'_a self) -> &'_a str;
    fn get_version<'_a>(&'_a self) -> &'_a HttpVersion;

    fn get_host<'_a>(&'_a self) -> Option<&'_a str>;
    fn get_headers<'_a>(&'_a self) -> &'_a HashMap<String, Vec<String>>;
    fn get_body<'_a>(&'_a self) -> &'_a [u8];

    fn clone(&self) -> Box<dyn HttpClient>;
}*/

#[derive(Debug, Clone)]
pub struct HttpClient{
    pub valid: bool,

    pub mpv_complete: bool,
    pub head_complete: bool,
    pub body_complete: bool,
    
    pub method: HttpMethod,
    pub path: String,
    pub version: HttpVersion,

    pub headers: HashMap<String, Vec<String>>,
    pub body: Vec<u8>,

    pub host: Option<String>, // should not be None in valid clients
    pub scheme: Option<String>,
}
impl HttpClient{
    pub fn reset(&mut self) { *self = Default::default() }
}
impl Default for HttpClient{
    fn default() -> Self {
        Self {
            valid: true,

            mpv_complete: false,
            head_complete: false,
            body_complete: false,

            method: HttpMethod::Unknown(None),
            path: String::new(),
            version: HttpVersion::Unknown(None),

            headers: HashMap::new(),
            body: Vec::new(),

            host: None,
            scheme: None,
        }
    }
}


pub trait HttpSocket{
    fn get_type(&self) -> HttpType;

    fn get_client<'_a>(&'_a self) -> &'_a HttpClient;
    fn read_client<'_a>(&'_a mut self) -> Pin<Box<dyn Future<Output = Result<&'_a HttpClient, std::io::Error>> + Send + '_a>>;
    fn read_until_complete<'_a>(&'_a mut self) -> Pin<Box<dyn Future<Output = Result<&'_a HttpClient, std::io::Error>> + Send + '_a>>;
    fn read_until_head_complete<'_a>(&'_a mut self) -> Pin<Box<dyn Future<Output = Result<&'_a HttpClient, std::io::Error>> + Send + '_a>>;

    fn add_header(&mut self, header: &str, value: &str);
    fn set_header(&mut self, header: &str, value: &str);
    fn del_header(&mut self, header: &str) -> Option<Vec<String>>;
    
    fn set_status(&mut self, code: u16, message: String);
    fn write<'a>(&'a mut self, body: &'a [u8]) -> Pin<Box<dyn Future<Output = Result<(), std::io::Error>> + Send + 'a>>;
    fn close<'a>(&'a mut self, body: &'a [u8]) -> Pin<Box<dyn Future<Output = Result<(), std::io::Error>> + Send + 'a>>;
    fn flush<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = Result<(), std::io::Error>> + Send + 'a>>;
}

// pub type DynHttpSocket = Box<dyn HttpSocket>;



#[derive(Debug, Clone)]
pub struct HttpResponse{
    pub valid: bool,

    pub vcs_complete: bool,
    pub head_complete: bool,
    pub body_complete: bool,

    pub version: HttpVersion,
    pub code: u16,
    pub status: String,

    pub headers: HashMap<String, Vec<String>>,
    pub body: Vec<u8>,
}
impl Default for HttpResponse{
    fn default() -> Self {
        Self {
            valid: true,

            vcs_complete: false,
            head_complete: false,
            body_complete: false,

            version: HttpVersion::Unknown(None),
            code: 0,
            status: String::new(),

            headers: HashMap::new(),
            body: Vec::new(),
        }
    }
}
impl HttpResponse{
    pub fn reset(&mut self){
        *self = Default::default();
    }
}

pub trait HttpRequest{
    fn get_type(&self) -> HttpType;

    fn add_header(&mut self, header: &str, value: &str);
    fn set_header(&mut self, header: &str, value: &str);
    fn del_header(&mut self, header: &str) -> Option<Vec<String>>;
    
    fn set_method(&mut self, method: HttpMethod);
    fn set_path(&mut self, path: String);

    fn write<'a>(&'a mut self, body: &'a [u8]) -> Pin<Box<dyn Future<Output = Result<(), std::io::Error>> + Send + 'a>>;
    fn send<'a>(&'a mut self, body: &'a [u8]) -> Pin<Box<dyn Future<Output = Result<(), std::io::Error>> + Send + 'a>>;
    fn flush<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = Result<(), std::io::Error>> + Send + 'a>>;

    fn get_response<'_a>(&'_a self) -> &'_a HttpResponse;
    fn read_response<'_a>(&'_a mut self) -> Pin<Box<dyn Future<Output = Result<&'_a HttpResponse, std::io::Error>> + Send + '_a>>;
    fn read_until_complete<'_a>(&'_a mut self) -> Pin<Box<dyn Future<Output = Result<&'_a HttpResponse, std::io::Error>> + Send + '_a>>;
    fn read_until_head_complete<'_a>(&'_a mut self) -> Pin<Box<dyn Future<Output = Result<&'_a HttpResponse, std::io::Error>> + Send + '_a>>;
}
