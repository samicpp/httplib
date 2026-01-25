use std::collections::HashMap;
use std::{pin::Pin};

use std::io;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader, ReadHalf, WriteHalf};
use crate::shared::HttpMethod;
use crate::shared::{HttpType, HttpVersion, ReadStream, Stream, WriteStream, server::{HttpClient, HttpSocket}};

pub const H2C_UPGRADE: &'static [u8] = b"HTTP/1.1 101 Switching Protocols\r\nConnection: Upgrade\r\nUpgrade: h2c\r\n\r\n";
pub const WS_UPGRADE: &'static [u8] = b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: ";



#[derive(Debug)]
pub struct Http1Socket<R: ReadStream, W: WriteStream>{
    pub netr: BufReader<R>,
    pub netw: W,

    pub client: Http1Client,
    pub line_buf: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Http1Client{
    pub valid: bool,

    pub mpv_complete: bool,
    pub head_complete: bool,
    pub body_complete: bool,
    
    pub method: HttpMethod,
    pub path: String,
    pub version: HttpVersion,

    pub headers: HashMap<String, Vec<String>>,
    pub body: Vec<u8>,
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
            netr,
            netw,
            client: Default::default(),
            line_buf: Vec::new(),
        }
    }

    pub async fn read_client(&mut self) -> io::Result<&Http1Client>{
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
                if mpv[2].eq_ignore_ascii_case("http/1.0") { HttpVersion::Http10 }
                else if mpv[2].eq_ignore_ascii_case("http/1.1") { HttpVersion::Http11 }
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
}

impl<R: ReadStream, W: WriteStream> HttpSocket for Http1Socket<R, W>{
    fn get_type() -> HttpType {
        HttpType::Http1
    }

    fn get_client(&self) -> &dyn HttpClient {
        &self.client
    }
    fn read_client(&'_ mut self) -> Pin<Box<dyn Future<Output = Result<&'_ dyn HttpClient, std::io::Error>> + '_>> {
        Box::pin(async move {
            self.read_client().await.and_then(|c| Ok(c as &dyn HttpClient))
        })
    }
}


impl HttpClient for Http1Client{
    fn get_method(&self) -> &HttpMethod {
        &self.method
    }
    fn get_path(&self) -> &str {
        &self.path
    }
    fn get_version(&self) -> &HttpVersion {
        &self.version
    }

    fn get_headers(&self) -> &HashMap<String, Vec<String>> {
        &self.headers
    }
    fn get_body(&self) -> &[u8] {
        &self.body
    }
}

impl Default for Http1Client{
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
        }
    }
}