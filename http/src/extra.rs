use std::pin::Pin;

use crate::{http1::server::Http1Socket, shared::{HttpType, ReadStream, WriteStream, HttpClient, HttpSocket}};

pub enum PolyHttpSocket<R: ReadStream, W: WriteStream>{
    Http1(Http1Socket<R, W>)
}

impl<R: ReadStream, W: WriteStream> HttpSocket for PolyHttpSocket<R, W>{
    fn get_type(&self) -> HttpType {
        match self {
            Self::Http1(_) => HttpType::Http1,
        }
    }

    fn get_client(&self) -> &HttpClient {
        match self{
            Self::Http1(h) => &h.client,
        }
    }
    fn read_client(&'_ mut self) -> Pin<Box<dyn Future<Output = Result<&'_ HttpClient, std::io::Error>> + Send + '_>> {
        Box::pin(async move {
            match self{
                Self::Http1(h) => h.read_client().await,
            }
        })
    }
    fn read_until_complete(&'_ mut self) -> Pin<Box<dyn Future<Output = Result<&'_ HttpClient, std::io::Error>> + Send + '_>> {
        Box::pin(async move {
            match self{
                Self::Http1(h) => h.read_until_complete().await,
            }
        })
    }
    fn read_until_head_complete(&'_ mut self) -> Pin<Box<dyn Future<Output = Result<&'_ HttpClient, std::io::Error>> + Send + '_>> {
        Box::pin(async move {
            match self{
                Self::Http1(h) => h.read_until_head_complete().await,
            }
        })
    }

    fn add_header(&mut self, header: &str, value: &str) { 
        match self {
            Self::Http1(h) => h.add_header(header, value),
        }
    }
    fn set_header(&mut self, header: &str, value: &str){ 
        match self {
            Self::Http1(h) => h.set_header(header, value),
        } 
    }
    fn del_header(&mut self, header: &str) -> Option<Vec<String>>{ 
        match self {
            Self::Http1(h) => h.del_header(header),
        }
    }

    fn set_status(&mut self, code: u16, message: String) {
        match self {
            Self::Http1(h) => {
                h.code = code;
                h.status = message;
            },
        }
    }
    fn write<'a>(&'a mut self, body: &'a [u8] ) -> Pin<Box<dyn Future<Output = Result<(), std::io::Error>> + Send + 'a>> {
        Box::pin(async move {
            match self{
                Self::Http1(h) => h.write(body).await,
            }
        })
    }
    fn close<'a>(&'a mut self, body: &'a [u8] ) -> Pin<Box<dyn Future<Output = Result<(), std::io::Error>> + Send + 'a>> {
        Box::pin(async move {
            match self{
                Self::Http1(h) => h.close(body).await,
            }
        })
    }
}
