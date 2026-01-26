use std::collections::HashMap;

use tokio::io::{BufReader, ReadHalf, WriteHalf};

use crate::shared::{HttpMethod, HttpVersion, ReadStream, Stream, WriteStream};



#[derive(Debug)]
pub struct Http1Request<R: ReadStream, W: WriteStream>{
    pub netr: BufReader<R>,
    pub netw: W,

    pub line_buf: Vec<u8>,

    pub sent_head: bool,
    pub closed: bool,
    
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

            line_buf: Vec::new(),
            
            sent_head: false,
            closed: false,

            path: String::new(),
            method: HttpMethod::Get,
            version: HttpVersion::Http11,
            headers: HashMap::new(),
        }
    }

    
}
