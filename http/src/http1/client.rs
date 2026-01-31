use std::{collections::HashMap, io, pin::Pin};

use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, ReadHalf, WriteHalf};

use crate::{http1::get_chunk, shared::{HttpMethod, HttpType, HttpVersion, ReadStream, Stream, WriteStream, HttpRequest, HttpResponse}};



#[derive(Debug)]
pub struct Http1Request<R: ReadStream, W: WriteStream>{
    pub netr: BufReader<R>,
    pub netw: W,

    pub response: HttpResponse,
    pub line_buf: Vec<u8>,

    pub sent_head: bool,
    pub sent: bool,
    
    pub path: String,
    pub method: HttpMethod,
    pub version: HttpVersion,
    pub headers: HashMap<String, Vec<String>>,
}


impl<S:Stream> Http1Request<ReadHalf<S>, WriteHalf<S>>{
    pub fn new(net: S, bufsize: usize) -> Self{
        let (netr, netw) = tokio::io::split(net);
        let bufr = BufReader::with_capacity(bufsize, netr);
        Self::with_split(bufr, netw)
    }
}
impl<R: ReadStream, W: WriteStream> Http1Request<R, W>{
    pub fn with_split(netr: BufReader<R>, netw: W) -> Self {
        Self {
            netr, netw,

            response: Default::default(),
            line_buf: Vec::new(),
            
            sent_head: false,
            sent: false,

            path: String::new(),
            method: HttpMethod::Get,
            version: HttpVersion::Http11,
            headers: HashMap::new(),
        }
    }

    pub fn add_header(&mut self, header: &str, value: &str) {
        if let Some(hs) = self.headers.get_mut(header) { hs.push(value.to_owned()); }
        else { self.headers.insert(header.to_owned(), vec![ value.to_owned() ]); }
    }
    pub fn set_header(&mut self, header: &str, value: &str){
        self.headers.insert(header.to_owned(), vec![ value.to_owned() ]);
    }
    pub fn del_header(&mut self, header: &str) -> Option<Vec<String>>{
        self.headers.remove(header)
    }

    pub async fn send_head(&mut self) -> io::Result<()> {
        if !self.sent_head && self.version == HttpVersion::Http09 {
            let head = format!("GET {}", &self.path);
            self.netw.write(head.as_bytes()).await?;
            self.sent_head = true;
            Ok(())
        }
        else if !self.sent_head{
            let headers = self.headers.iter().map(|(h,vs)|vs.iter().map(|v| format!("{}: {}\r\n", h, v)).collect::<String>()).collect::<String>();
            let head = format!(
                "{} {} {}\r\n{}\r\n", 
                match &self.method { HttpMethod::Unknown(Some(s)) => s.to_owned(), v => format!("{}", v)},
                &self.path,
                match &self.version { HttpVersion::Unknown(Some(s)) => s.to_owned(), v => format!("{}", v)},
                headers,
            );
            
            self.netw.write(head.as_bytes()).await?;
            self.sent_head = true;

            Ok(())
        }
        else{
            Err(io::Error::new(io::ErrorKind::NotConnected, "connection closed"))
        }
    }

    pub async fn write(&mut self, body: &[u8]) -> io::Result<()> {
        if !self.sent && self.version == HttpVersion::Http09 {
            if !self.sent_head { self.send_head().await? }
            Ok(())
        }
        else if !self.sent{
            if !self.sent_head{
                self.headers.insert("Transfer-Encoding".to_owned(), vec!["chunked".to_owned()]);
                self.send_head().await?;
            }
            self.netw.write(&get_chunk(body)).await?;
            Ok(())
        }
        else{
            Err(io::Error::new(io::ErrorKind::NotConnected, "connection closed"))
        }
    }
    pub async fn send(&mut self, body: &[u8]) -> io::Result<()> {
        if !self.sent && self.version == HttpVersion::Http09 {
            if !self.sent_head { self.send_head().await? }
            self.sent = true;
            Ok(())
        }
        else if !self.sent_head{
            self.headers.insert("Content-Length".to_owned(), vec![body.len().to_string()]);
            self.send_head().await?;
            self.netw.write(body).await?;
            self.sent = true;
            Ok(())
        }
        else if !self.sent{
            self.netw.write(&get_chunk(body)).await?;
            self.netw.write(b"0\r\n\r\n").await?;
            Ok(())
        }
        else{
            Err(io::Error::new(io::ErrorKind::NotConnected, "connection closed"))
        }
    }
    pub async fn flush(&mut self) -> io::Result<()> {
        self.netw.flush().await
    }

    pub async fn read_response(&mut self) -> io::Result<&HttpResponse> {
        self.line_buf.clear();

        if !self.response.valid {

        }
        else if !self.response.vcs_complete && self.version != HttpVersion::Http09 {
            let _ = self.netr.read_until(b'\n', &mut self.line_buf).await?;

            let fullstr = String::from_utf8_lossy(&self.line_buf);
            let vcs: Vec<&str> = fullstr.splitn(3, ' ').collect();
            
            if vcs.len() != 3{
                self.response.valid = false;
            }
            else{
                self.response.version = // should always be same version as client
                if vcs[0].eq_ignore_ascii_case("http/1.0") { HttpVersion::Http10 }
                else if vcs[0].eq_ignore_ascii_case("http/1.1") { HttpVersion::Http11 }
                else { HttpVersion::Unknown(Some(vcs[2].to_owned())) };
                self.response.code = vcs[1].parse().unwrap_or(0);
                self.response.status = vcs[2].to_owned();
            }

            self.response.vcs_complete = true;
        }
        else if !self.response.head_complete && self.version != HttpVersion::Http09 {
            let _ = self.netr.read_until(b'\n', &mut self.line_buf).await?;
            let fullstr = String::from_utf8_lossy(&self.line_buf);
            let hv: Vec<&str> = fullstr.splitn(2, ':').map(|e|e.trim()).collect();

            if fullstr.trim().is_empty(){
                self.response.head_complete = true;
            }
            else if hv.len() == 1 {
                self.response.valid = false;
            }
            else{
                if let Some(hs) = self.response.headers.get_mut(&hv[0].to_ascii_lowercase()) { hs.push(hv[1].to_owned()); }
                else { self.response.headers.insert(hv[0].to_ascii_lowercase(), vec![ hv[1].to_owned() ]); }
            }
        }
        else if !self.response.body_complete {
            // TODO: http10 read until EOF
            if let Some(te) = self.response.headers.get("transfer-encoding") && te[0].contains("chunked") {
                let _ = self.netr.read_until(b'\n', &mut self.line_buf).await?;
                let string = String::from_utf8_lossy(&self.line_buf);
                let len = match usize::from_str_radix(string.trim(), 16) { Ok(s) => s, Err(_) => 0, };

                if len == 0{
                    self.response.body_complete = true;
                }
                else{
                    let ol = self.response.body.len();
                    self.response.body.resize(self.response.body.len() + len + 1, 0);  // len + 1 allows LF seperated chunks (technically should be CRLF)
                    self.netr.read_exact(&mut self.response.body[ol..]).await?;
                }
            }
            else if let Some(cl) = self.response.headers.get("content-length") && let Ok(len) = cl[0].parse::<usize>(){
                self.response.body.resize(len, 0);
                self.netr.read_exact(&mut self.response.body).await?;
                self.response.body_complete = true;
            }
            else{
                self.response.body_complete = true;
            }
        }
        Ok(&self.response)
    }
    pub async fn read_until_complete(&mut self) -> io::Result<&HttpResponse>{
        while !self.read_response().await?.body_complete {}
        Ok(&self.response)
    }
    pub async fn read_until_head_complete(&mut self) -> io::Result<&HttpResponse>{
        while !self.read_response().await?.head_complete {}
        Ok(&self.response)
    }

    pub fn reset(&mut self){
        self.response.reset();
        self.method = HttpMethod::Get;
        self.path = String::new();
        self.version = HttpVersion::Http11;
        self.headers.clear();
        self.sent_head = false;
        self.sent = false;
    }
}
impl<R: ReadStream, W: WriteStream> HttpRequest for Http1Request<R, W>{
    fn get_type(&self) -> HttpType {
        HttpType::Http1
    }

    fn get_response(&self) -> &HttpResponse {
        &self.response
    }
    fn read_response(&'_ mut self) -> Pin<Box<dyn Future<Output = Result<&'_ HttpResponse, std::io::Error>> + Send + '_>> {
        Box::pin(async move {
            self.read_response().await
        })
    }
    fn read_until_complete(&'_ mut self) -> Pin<Box<dyn Future<Output = Result<&'_ HttpResponse, std::io::Error>> + Send + '_>> {
        Box::pin(async move {
            self.read_until_complete().await
        })
    }
    fn read_until_head_complete(&'_ mut self) -> Pin<Box<dyn Future<Output = Result<&'_ HttpResponse, std::io::Error>> + Send + '_>> {
        Box::pin(async move {
            self.read_until_head_complete().await
        })
    }


    fn add_header(&mut self, header: &str, value: &str) { self.add_header(header, value) }
    fn set_header(&mut self, header: &str, value: &str){ self.set_header(header, value) }
    fn del_header(&mut self, header: &str) -> Option<Vec<String>>{ self.del_header(header) }

    fn set_method(&mut self, method: HttpMethod) {
        self.method = method;
    }
    fn set_path(&mut self, path: String) {
        self.path = path;
    }
    fn write<'a>(&'a mut self, body: &'a [u8] ) -> Pin<Box<dyn Future<Output = Result<(), std::io::Error>> + Send + 'a>> {
        Box::pin(async move {
            self.write(body).await
        })
    }
    fn send<'a>(&'a mut self, body: &'a [u8] ) -> Pin<Box<dyn Future<Output = Result<(), std::io::Error>> + Send + 'a>> {
        Box::pin(async move {
            self.send(body).await
        })
    }
    fn flush<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = Result<(), std::io::Error>> + Send + 'a>> {
        Box::pin(async move{
            self.flush().await
        })
    }
}

