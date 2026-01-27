use std::collections::HashMap;
use std::{pin::Pin};

use std::io;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, ReadHalf, WriteHalf};
use crate::http1::get_chunk;
use crate::shared::HttpMethod;
use crate::shared::{HttpType, HttpVersion, ReadStream, Stream, WriteStream, HttpClient, HttpSocket};


pub const H2C_UPGRADE: &'static [u8] = b"HTTP/1.1 101 Switching Protocols\r\nConnection: Upgrade\r\nUpgrade: h2c\r\n\r\n";
pub const WS_UPGRADE: &'static [u8] = b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: ";



#[derive(Debug)]
pub struct Http1Socket<R: ReadStream, W: WriteStream>{
    pub netr: BufReader<R>,
    pub netw: W,

    pub client: HttpClient,
    pub line_buf: Vec<u8>,

    pub sent_head: bool,
    pub closed: bool,
    
    pub code: u16,
    pub status: String,
    pub headers: HashMap<String, Vec<String>>,
}


impl<S:Stream> Http1Socket<ReadHalf<S>, WriteHalf<S>>{
    pub fn new(net: S, bufsize: usize) -> Self{
        let (netr, netw) = tokio::io::split(net);
        let bufr = BufReader::with_capacity(bufsize, netr);
        Self::with_split(bufr, netw)
    }
}

impl<R: ReadStream, W: WriteStream> Http1Socket<R, W>{
    pub fn with_split(netr: BufReader<R>, netw: W) -> Self {
        Self {
            netr, netw,

            client: Default::default(),
            line_buf: Vec::new(),
            
            sent_head: false,
            closed: false,

            code: 200,
            status: "OK".to_string(),
            headers: HashMap::new(),
        }
    }


    pub async fn read_client(&mut self) -> io::Result<&HttpClient>{
        self.line_buf.clear();

        if !self.client.valid {

        }
        else if !self.client.mpv_complete{
            let _ = self.netr.read_until(b'\n', &mut self.line_buf).await?;

            let fullstr = String::from_utf8_lossy(&self.line_buf);
            let mpv: Vec<&str> = fullstr.splitn(3, ' ').collect();

            if mpv.len() < 2{
                self.client.valid = false;
            }
            else if mpv.len() == 2{
                self.client.method = HttpMethod::Get;
                self.client.path = mpv[1].to_owned();
                self.client.version = HttpVersion::Http09;

                self.client.head_complete = true;
                self.client.body_complete = true;
            }
            else{
                self.client.method = HttpMethod::from(mpv[0]);
                self.client.path = mpv[1].to_owned();
                self.client.version = 
                if mpv[2].trim().eq_ignore_ascii_case("http/1.0") { HttpVersion::Http10 }
                else if mpv[2].trim().eq_ignore_ascii_case("http/1.1") { HttpVersion::Http11 }
                else { HttpVersion::Unknown(Some(mpv[2].to_owned())) };
            }

            self.client.mpv_complete = true;
        }
        else if !self.client.head_complete{
            let _ = self.netr.read_until(b'\n', &mut self.line_buf).await?;
            let fullstr = String::from_utf8_lossy(&self.line_buf);
            let hv: Vec<&str> = fullstr.splitn(2, ':').map(|e|e.trim()).collect();

            if fullstr.trim().is_empty(){
                self.client.head_complete = true;
            }
            else if hv.len() == 1 {
                self.client.valid = false;
            }
            else{
                if let Some(hs) = self.client.headers.get_mut(&hv[0].to_ascii_lowercase()) { hs.push(hv[1].to_owned()); }
                else { self.client.headers.insert(hv[0].to_ascii_lowercase(), vec![ hv[1].to_owned() ]); }
            }
        }
        else if !self.client.body_complete{
            // TODO: http10 read until EOF
            if let Some(te) = self.client.headers.get("transfer-encoding") && te[0].contains("chunked") {
                let _ = self.netr.read_until(b'\n', &mut self.line_buf).await?;
                let string = String::from_utf8_lossy(&self.line_buf);
                let len = match usize::from_str_radix(string.trim(), 16) { Ok(s) => s, Err(_) => 0, };

                if len == 0{
                    self.client.body_complete = true;
                }
                else{
                    let ol = self.client.body.len();
                    self.client.body.resize(self.client.body.len() + len + 1, 0);  // len + 1 allows LF seperated chunks (technically should be CRLF)
                    self.netr.read_exact(&mut self.client.body[ol..]).await?;
                }
            }
            else if let Some(cl) = self.client.headers.get("content-length") && let Ok(len) = cl[0].parse::<usize>(){
                self.client.body.resize(len, 0);
                self.netr.read_exact(&mut self.client.body).await?;
                self.client.body_complete = true;
            }
            else{
                self.client.body_complete = true;
            }
        }

        Ok(&self.client)
    }
    pub async fn read_until_complete(&mut self) -> io::Result<&HttpClient>{
        while !self.read_client().await?.body_complete {}
        Ok(&self.client)
    }
    pub async fn read_until_head_complete(&mut self) -> io::Result<&HttpClient>{
        while !self.read_client().await?.head_complete {}
        Ok(&self.client)
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
        if !self.sent_head && self.client.version == HttpVersion::Http09 {
            self.sent_head = true;
            Ok(())
        }
        else if !self.sent_head{
            let headers = self.headers.iter().map(|(h,vs)|vs.iter().map(|v| format!("{}: {}\r\n", h, v)).collect::<String>()).collect::<String>();
            let head = format!(
                "{} {} {}\r\n{}\r\n", 
                match &self.client.version { HttpVersion::Unknown(Some(s)) => s.to_owned(), v => format!("{}", v)},
                self.code,
                &self.status,
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

    pub async fn write(&mut self, body: &[u8]) -> io::Result<()>{
        if !self.closed && self.client.version == HttpVersion::Http09 {
            if !self.sent_head { self.send_head().await? }
            self.netw.write(body).await?;
            Ok(())
        }
        else if !self.closed{
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
    pub async fn close(&mut self, body: &[u8]) -> io::Result<()>{
        if !self.sent_head{
            self.headers.insert("Content-Length".to_owned(), vec![body.len().to_string()]);
            self.send_head().await?;
            self.netw.write(body).await?;
            self.closed = true;
            Ok(())
        }
        else if !self.closed && self.client.version == HttpVersion::Http09 {
            self.netw.write(body).await?;
            Ok(())
        }
        else if !self.closed{
            self.netw.write(&get_chunk(body)).await?;
            self.netw.write(b"0\r\n\r\n").await?;
            self.closed = true;
            Ok(())
        }
        else{
            Err(io::Error::new(io::ErrorKind::NotConnected, "connection closed"))
        }
    }

    pub fn reset(&mut self){
        self.client.reset();
        self.code = 200;
        self.status = "OK".to_owned();
        self.headers.clear();
        self.sent_head = false;
        self.closed = false;
    }
}

impl<R: ReadStream, W: WriteStream> HttpSocket for Http1Socket<R, W>{
    fn get_type(&self) -> HttpType {
        HttpType::Http1
    }

    fn get_client(&self) -> &HttpClient {
        &self.client
    }
    fn read_client(&'_ mut self) -> Pin<Box<dyn Future<Output = Result<&'_ HttpClient, std::io::Error>> + Send + '_>> {
        Box::pin(async move {
            self.read_client().await
        })
    }
    fn read_until_complete(&'_ mut self) -> Pin<Box<dyn Future<Output = Result<&'_ HttpClient, std::io::Error>> + Send + '_>> {
        Box::pin(async move {
            self.read_until_complete().await
        })
    }
    fn read_until_head_complete(&'_ mut self) -> Pin<Box<dyn Future<Output = Result<&'_ HttpClient, std::io::Error>> + Send + '_>> {
        Box::pin(async move {
            self.read_until_head_complete().await
        })
    }

    fn add_header(&mut self, header: &str, value: &str) { self.add_header(header, value) }
    fn set_header(&mut self, header: &str, value: &str){ self.set_header(header, value) }
    fn del_header(&mut self, header: &str) -> Option<Vec<String>>{ self.del_header(header) }

    fn set_status(&mut self, code: u16, message: String) {
        self.code = code;
        self.status = message;
    }
    fn write<'a>(&'a mut self, body: &'a [u8] ) -> Pin<Box<dyn Future<Output = Result<(), std::io::Error>> + Send + 'a>> {
        Box::pin(async move {
            self.write(body).await
        })
    }
    fn close<'a>(&'a mut self, body: &'a [u8] ) -> Pin<Box<dyn Future<Output = Result<(), std::io::Error>> + Send + 'a>> {
        Box::pin(async move {
            self.close(body).await
        })
    }
}

