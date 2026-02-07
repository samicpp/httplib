use std::collections::VecDeque;

pub mod huffman;
pub mod encoder;
pub mod decoder;


pub static STATIC_TABLE: &'static [(&'static [u8], &'static [u8]); 61] = &[
    (b":authority", b""),
    (b":method", b"GET"),
    (b":method", b"POST"),
    (b":path", b"/"),
    (b":path", b"/index.html"),
    (b":scheme", b"http"),
    (b":scheme", b"https"),
    (b":status", b"200"),
    (b":status", b"204"),
    (b":status", b"206"),
    (b":status", b"304"),
    (b":status", b"400"),
    (b":status", b"404"),
    (b":status", b"500"),
    (b"accept-", b""),
    (b"accept-encoding", b"gzip, deflate"),
    (b"accept-language", b""),
    (b"accept-ranges", b""),
    (b"accept", b""),
    (b"access-control-allow-origin", b""),
    (b"age", b""),
    (b"allow", b""),
    (b"authorization", b""),
    (b"cache-control", b""),
    (b"content-disposition", b""),
    (b"content-encoding", b""),
    (b"content-language", b""),
    (b"content-length", b""),
    (b"content-location", b""),
    (b"content-range", b""),
    (b"content-type", b""),
    (b"cookie", b""),
    (b"date", b""),
    (b"etag", b""),
    (b"expect", b""),
    (b"expires", b""),
    (b"from", b""),
    (b"host", b""),
    (b"if-match", b""),
    (b"if-modified-since", b""),
    (b"if-none-match", b""),
    (b"if-range", b""),
    (b"if-unmodified-since", b""),
    (b"last-modified", b""),
    (b"link", b""),
    (b"location", b""),
    (b"max-forwards", b""),
    (b"proxy-authenticate", b""),
    (b"proxy-authorization", b""),
    (b"range", b""),
    (b"referer", b""),
    (b"refresh", b""),
    (b"retry-after", b""),
    (b"server", b""),
    (b"set-cookie", b""),
    (b"strict-transport-security", b""),
    (b"transfer-encoding", b""),
    (b"user-agent", b""),
    (b"vary", b""),
    (b"via", b""),
    (b"www-authenticate", b""),
];

pub struct DynamicTable {
    pub size: usize,
    pub table_size: usize,
    pub table: VecDeque<(Vec<u8>, Vec<u8>)>,
}
impl DynamicTable {
    pub fn new(table_size: usize) -> Self {
        Self {
            size: 0,
            table_size,
            table: VecDeque::new(),
        }
    }
    pub fn add(&mut self, header: (Vec<u8>, Vec<u8>)) {
        self.size += header.0.len() + header.1.len() + 32;
        self.table.push_front(header);
        self.evict();
    }
    pub fn evict(&mut self) {
        while self.size > self.table_size {
            let (name, value) = 
            match self.table.pop_back() {
                None => break,
                Some(header) => header
            };
            self.size -= name.len() + value.len() + 32;
        }
    }
    pub fn resize(&mut self, new_size: usize) {
        self.table_size = new_size;
        self.evict();
    }
}
