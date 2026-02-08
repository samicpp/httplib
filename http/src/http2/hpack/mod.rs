use std::collections::VecDeque;

pub mod huffman;
pub mod encoder;
pub mod decoder;

// https://datatracker.ietf.org/doc/html/rfc7541


// Appendix A
pub const STATIC_TABLE: &'static [(&'static [u8], &'static [u8]); 61] = &[
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

#[derive(Debug, Clone)]
pub enum StaticTable<'a>{
    Borrow(&'a [(&'a [u8], &'a [u8])]),
    Owned(Vec<(Vec<u8>, Vec<u8>)>),
}
impl<'a> StaticTable<'a>{
    pub fn is_owned(&self) -> bool {
        if let Self::Owned(_) = self { true }
        else { false }
    }
    pub fn is_borrow(&self) -> bool {
        if let Self::Borrow(_) = self { true }
        else { false }
    }
    pub fn to_owned(&mut self) {
        match self {
            Self::Borrow(b) => {
                let clone = b.iter().map(|(h, v)| (h.to_vec(), v.to_vec())).collect();
                *self = Self::Owned(clone);
            },
            _ => (),
        }
    }
    pub fn into_owned(self) -> Self {
        match self {
            Self::Borrow(b) => {
                let clone = b.iter().map(|(h, v)| (h.to_vec(), v.to_vec())).collect();
                Self::Owned(clone)
            },
            Self::Owned(v) => Self::Owned(v),
        }
    }

    pub fn get(&self, index: usize) -> Option<(&[u8], &[u8])> {
        match self {
            Self::Borrow(b) => {
                b.get(index).map(|t| *t)
            }
            Self::Owned(v) => {
                v.get(index).map(|(h, v)| (h.as_slice(), v.as_slice()))
            }
        }
    }
    pub fn len(&self) -> usize {
        match self {
            Self::Borrow(b) => b.len(),
            Self::Owned(o) => o.len(),
        }
    }

    pub fn iter(&'a self) -> StaticTableIterator<'a> {
        StaticTableIterator { cur: 0, table: self }
    }
}
impl<'a> From<&'a [(&'a [u8], &'a [u8])]> for StaticTable<'a> {
    fn from(value: &'a [(&'a [u8], &'a [u8])]) -> Self {
        Self::Borrow(value)
    }
}
impl From<Vec<(Vec<u8>, Vec<u8>)>> for StaticTable<'static> {
    fn from(value: Vec<(Vec<u8>, Vec<u8>)>) -> Self {
        Self::Owned(value)
    }
}
impl<'a> Into<Vec<(Vec<u8>, Vec<u8>)>> for StaticTable<'a> {
    fn into(self) -> Vec<(Vec<u8>, Vec<u8>)> {
        match self{
            Self::Borrow(b) => b.iter().map(|(h, v)| (h.to_vec(), v.to_vec())).collect(),
            Self::Owned(v) => v,
        }
    }
}

pub struct StaticTableIterator<'a>{
    cur: usize,
    table: &'a StaticTable<'a>,
}
impl<'a> Iterator for StaticTableIterator<'a> {
    type Item = (&'a [u8], &'a [u8]);

    fn next(&mut self) -> Option<Self::Item> {
        self.cur += 1;
        self.table.get(self.cur - 1)
    }
}

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


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeaderType{
    Lookup,
    Indexed,
    NotIndexed,
    NeverIndexed,
    TableSizeChange,
}


// no reason for public fields
pub struct Biterator<'a, I: Iterator<Item = &'a u8>> {
    buff_iter: I,
    current: Option<&'a u8>,
    pos: u8,
}
impl<'a, I: Iterator<Item = &'a u8>> Biterator<'a, I>{
    pub fn new(iter: I) -> Biterator<'a, I> {
        Self {
            buff_iter: iter,
            current: None,
            pos: 7,
        }
    }
}
impl<'a, I: Iterator<Item = &'a u8>> Iterator for Biterator<'a, I> {
    type Item = bool;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_none() {
            self.current = Some(self.buff_iter.next()?);
            self.pos = 7;
        }

        let byte = unsafe{ *self.current.unwrap_unchecked() };
        let bit = byte & (1 << self.pos) != 0;
        
        if self.pos == 0 {
            self.current = None;
        }
        else {
            self.pos -= 1;
        }

        Some(bit)
    }
}
