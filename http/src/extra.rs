use std::pin::Pin;

use crate::{http1::{client::Http1Request, server::Http1Socket}, shared::{HttpClient, HttpMethod, HttpRequest, HttpResponse, HttpSocket, HttpType, LibError, ReadStream, WriteStream}};

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
    fn read_client(&'_ mut self) -> Pin<Box<dyn Future<Output = Result<&'_ HttpClient, LibError>> + Send + '_>> {
        Box::pin(async move {
            match self{
                Self::Http1(h) => h.read_client().await,
            }
        })
    }
    fn read_until_complete(&'_ mut self) -> Pin<Box<dyn Future<Output = Result<&'_ HttpClient, LibError>> + Send + '_>> {
        Box::pin(async move {
            match self{
                Self::Http1(h) => h.read_until_complete().await,
            }
        })
    }
    fn read_until_head_complete(&'_ mut self) -> Pin<Box<dyn Future<Output = Result<&'_ HttpClient, LibError>> + Send + '_>> {
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
    fn write<'a>(&'a mut self, body: &'a [u8] ) -> Pin<Box<dyn Future<Output = Result<(), LibError>> + Send + 'a>> {
        Box::pin(async move {
            match self{
                Self::Http1(h) => h.write(body).await,
            }
        })
    }
    fn close<'a>(&'a mut self, body: &'a [u8] ) -> Pin<Box<dyn Future<Output = Result<(), LibError>> + Send + 'a>> {
        Box::pin(async move {
            match self{
                Self::Http1(h) => h.close(body).await,
            }
        })
    }
    fn flush<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = Result<(), LibError>> + Send + 'a>> {
        Box::pin(async move{
            match self{
                Self::Http1(h) => h.flush().await,
            }
        })
    }
}

impl<R: ReadStream, W: WriteStream> From<Http1Socket<R, W>> for PolyHttpSocket<R, W> {
    fn from(value: Http1Socket<R, W>) -> Self {
        Self::Http1(value)
    }
}


pub enum PolyHttpRequest<R: ReadStream, W: WriteStream>{
    Http1(Http1Request<R, W>)
}

impl<R: ReadStream, W: WriteStream> HttpRequest for PolyHttpRequest<R, W>{
    fn get_type(&self) -> HttpType {
        match self {
            Self::Http1(_) => HttpType::Http1,
        }
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
    
    fn set_method(&mut self, method: HttpMethod){
        match self {
            Self::Http1(h) => h.set_method(method),
        }
    }
    fn set_scheme(&mut self, scheme: String) {
        match self {
            Self::Http1(h) => h.set_scheme(scheme),
        }
    }
    fn set_path(&mut self, method: String){
        match self {
            Self::Http1(h) => h.set_path(method),
        }
    }
    fn set_host(&mut self, host: String) {
        match self {
            Self::Http1(h) => h.set_host(host),
        }
    }

    fn write<'a>(&'a mut self, body: &'a [u8]) -> Pin<Box<dyn Future<Output = Result<(), LibError>> + Send + 'a>> {
        Box::pin(async move{
            match self {
                Self::Http1(h) => h.write(body).await,
            }
        })
    }
    fn send<'a>(&'a mut self, body: &'a [u8]) -> Pin<Box<dyn Future<Output = Result<(), LibError>> + Send + 'a>> {
        Box::pin(async move{
            match self {
                Self::Http1(h) => h.send(body).await,
            }
        })
    }
    fn flush<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = Result<(), LibError>> + Send + 'a>> {
        Box::pin(async move{
            match self {
                Self::Http1(h) => h.flush().await,
            }
        })
    }

    fn get_response<'_a>(&'_a self) -> &'_a HttpResponse {
        match self {
            Self::Http1(h) => h.get_response(),
        }
    }
    fn read_response<'_a>(&'_a mut self) -> Pin<Box<dyn Future<Output = Result<&'_a HttpResponse, LibError>> + Send + '_a>> {
        Box::pin(async move{
            match self {
                Self::Http1(h) => h.read_response().await,
            }
        })
    }
    fn read_until_complete<'_a>(&'_a mut self) -> Pin<Box<dyn Future<Output = Result<&'_a HttpResponse, LibError>> + Send + '_a>> {
        Box::pin(async move{
            match self {
                Self::Http1(h) => h.read_until_complete().await,
            }
        })
    }
    fn read_until_head_complete<'_a>(&'_a mut self) -> Pin<Box<dyn Future<Output = Result<&'_a HttpResponse, LibError>> + Send + '_a>> {
        Box::pin(async move{
            match self {
                Self::Http1(h) => h.read_until_head_complete().await,
            }
        })
    }
}

impl<R: ReadStream, W: WriteStream> From<Http1Request<R, W>> for PolyHttpRequest<R, W>{
    fn from(value: Http1Request<R, W>) -> Self {
        Self::Http1(value)
    }
}
