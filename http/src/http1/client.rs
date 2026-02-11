use std::{collections::HashMap, pin::Pin};

use rand::Rng;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, ReadHalf, WriteHalf};

use crate::{http1::get_chunk, shared::{HttpMethod, HttpRequest, HttpResponse, HttpType, HttpVersion, LibError, LibResult, ReadStream, Stream, WriteStream}, websocket::socket::{MAGIC, WebSocket}};

use base64::{Engine, engine::general_purpose::STANDARD as b64std};

use sha1::{Sha1, Digest};


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

            path: "/".to_owned(),
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

    pub async fn send_head(&mut self) -> LibResult<()> {
        if !self.sent_head && self.version == HttpVersion::Http09 {
            let head = format!("GET {}\r\n", &self.path);
            self.netw.write_all(head.as_bytes()).await?;
            self.sent_head = true;
            self.response.vcs_complete = true;
            self.response.head_complete = true;
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
            
            self.netw.write_all(head.as_bytes()).await?;
            self.sent_head = true;

            Ok(())
        }
        else{
            Err(LibError::ConnectionClosed)
        }
    }

    pub async fn write(&mut self, body: &[u8]) -> LibResult<()> {
        if !self.sent && self.version == HttpVersion::Http09 {
            if !self.sent_head { self.send_head().await? }
            Ok(())
        }
        else if !self.sent{
            if !self.sent_head{
                self.headers.insert("Transfer-Encoding".to_owned(), vec!["chunked".to_owned()]);
                self.send_head().await?;
            }
            self.netw.write_all(&get_chunk(body)).await?;
            Ok(())
        }
        else{
            Err(LibError::ConnectionClosed)
        }
    }
    pub async fn send(&mut self, body: &[u8]) -> LibResult<()> {
        if !self.sent && self.version == HttpVersion::Http09 {
            if !self.sent_head { self.send_head().await? }
            self.sent = true;
            Ok(())
        }
        else if !self.sent_head{
            self.headers.insert("Content-Length".to_owned(), vec![body.len().to_string()]);
            self.send_head().await?;
            self.netw.write_all(body).await?;
            self.sent = true;
            Ok(())
        }
        else if !self.sent{
            self.netw.write_all(&get_chunk(body)).await?;
            self.netw.write_all(b"0\r\n\r\n").await?;
            Ok(())
        }
        else{
            Err(LibError::ConnectionClosed)
        }
    }
    pub async fn flush(&mut self) -> LibResult<()> {
        self.netw.flush().await.map_err(|e| e.into())
    }

    pub async fn read_response(&mut self) -> LibResult<&HttpResponse> {
        self.line_buf.clear();

        if !self.response.valid {

        }
        else if !self.response.vcs_complete && self.version != HttpVersion::Http09 {
            let _ = self.netr.read_until(b'\n', &mut self.line_buf).await?;

            let fullstr = String::from_utf8_lossy(&self.line_buf);
            let fullstr = fullstr.trim_end_matches(['\r', '\n']);
            let vcs: Vec<&str> = fullstr.splitn(3, ' ').collect();
            
            if vcs.len() != 3{
                self.response.valid = false;
            }
            else{
                self.response.version = // should always be same version as client
                if vcs[0].eq_ignore_ascii_case("http/1.0") { HttpVersion::Http10 }
                else if vcs[0].eq_ignore_ascii_case("http/1.1") { HttpVersion::Http11 }
                else { HttpVersion::Unknown(Some(vcs[0].to_owned())) };
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
            if let Some(te) = self.response.headers.get("transfer-encoding") && te[0].contains("chunked") {
                let _ = self.netr.read_until(b'\n', &mut self.line_buf).await?;
                let string = String::from_utf8_lossy(&self.line_buf);
                let len = match usize::from_str_radix(string.trim(), 16) { Ok(s) => s, Err(_) => 0, };

                if len == 0{
                    self.response.body_complete = true;
                }
                else{
                    let ol = self.response.body.len();
                    self.response.body.resize(self.response.body.len() + len, 0);
                    self.netr.read_exact(&mut self.response.body[ol..]).await?;
                    self.netr.read_until(b'\n', &mut self.line_buf).await?;
                }
            }
            else if let Some(cl) = self.response.headers.get("content-length") && let Ok(len) = cl[0].parse::<usize>(){
                self.response.body.resize(len, 0);
                self.netr.read_exact(&mut self.response.body).await?;
                self.response.body_complete = true;
            }
            else if self.response.version == HttpVersion::Http10 || self.response.version == HttpVersion::Http09 {
                self.netr.read_to_end(&mut self.response.body).await?;
                self.response.body_complete = true;
            }
            else{
                self.response.body_complete = true;
            }
        }
        Ok(&self.response)
    }
    pub async fn read_until_complete(&mut self) -> LibResult<&HttpResponse>{
        while self.response.valid && !self.response.body_complete { let _ = self.read_response().await?; }
        Ok(&self.response)
    }
    pub async fn read_until_head_complete(&mut self) -> LibResult<&HttpResponse>{
        while self.response.valid && !self.response.head_complete { let _ = self.read_response().await?; }
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

    pub async fn websocket_upgrade(&mut self, key: &[u8]) -> LibResult<String> {
        if key.len() != 16 { return Err(LibError::Invalid); }

        let wskey = b64std.encode(key);
        
        self.set_header("Connection", "upgrade");
        self.set_header("Upgrade", "websocket");
        self.set_header("Sec-WebSocket-Version", "13");
        self.set_header("Sec-WebSocket-Key", &wskey);

        self.send(b"").await?;
        Ok(wskey)
    }
    pub fn websocket_direct(self) -> WebSocket<BufReader<R>, W> {
        WebSocket::with_split(self.netr, self.netw)
    }
    pub async fn websocket_unchecked(mut self) -> LibResult<WebSocket<BufReader<R>, W>> {
        let mut key = [0; 16];
        rand::rng().fill(&mut key);
        let _ = self.websocket_upgrade(&key).await?;
        
        Ok(self.websocket_direct())
    }
    pub async fn websocket_lazy(mut self) -> LibResult<WebSocket<BufReader<R>, W>>{
        let mut key = [0; 16];
        rand::rng().fill(&mut key);
        let _ = self.websocket_upgrade(&key).await?;
        
        let res = self.read_until_head_complete().await?;
        if res.code != 101 {
            Err(LibError::NotAccepted)
        }
        else {
            Ok(self.websocket_direct())
        }
    }
    pub async fn websocket_strict(mut self) -> LibResult<WebSocket<BufReader<R>, W>>{
        let mut key = [0; 16];
        rand::rng().fill(&mut key);
        let bkey = self.websocket_upgrade(&key).await?;

        let mut sha = Sha1::new();
        sha.update(bkey.as_bytes());
        sha.update(MAGIC);
        let acckey = b64std.encode(sha.finalize());
        
        let res = self.read_until_head_complete().await?;
        if res.code != 101 {
            Err(LibError::NotAccepted)
        }
        else if let Some(reskey) = res.headers.get("sec-websocket-accept") && reskey[0] == acckey {
            Ok(self.websocket_direct())
        }
        else {
            Err(LibError::InvalidUpgrade)
        }
    }
}
impl<R: ReadStream, W: WriteStream> HttpRequest for Http1Request<R, W>{
    fn get_type(&self) -> HttpType {
        HttpType::Http1
    }

    fn get_response(&self) -> &HttpResponse {
        &self.response
    }
    fn read_response(&'_ mut self) -> Pin<Box<dyn Future<Output = Result<&'_ HttpResponse, LibError>> + Send + '_>> {
        Box::pin(async move {
            self.read_response().await
        })
    }
    fn read_until_complete(&'_ mut self) -> Pin<Box<dyn Future<Output = Result<&'_ HttpResponse, LibError>> + Send + '_>> {
        Box::pin(async move {
            self.read_until_complete().await
        })
    }
    fn read_until_head_complete(&'_ mut self) -> Pin<Box<dyn Future<Output = Result<&'_ HttpResponse, LibError>> + Send + '_>> {
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
    fn write<'a>(&'a mut self, body: &'a [u8] ) -> Pin<Box<dyn Future<Output = Result<(), LibError>> + Send + 'a>> {
        Box::pin(async move {
            self.write(body).await
        })
    }
    fn send<'a>(&'a mut self, body: &'a [u8] ) -> Pin<Box<dyn Future<Output = Result<(), LibError>> + Send + 'a>> {
        Box::pin(async move {
            self.send(body).await
        })
    }
    fn flush<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = Result<(), LibError>> + Send + 'a>> {
        Box::pin(async move{
            self.flush().await
        })
    }
}

