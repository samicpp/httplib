use std::io;

use crate::http2::hpack::{DynamicTable, HeaderType, STATIC_TABLE, StaticTable, huffman::Huffman};

#[derive(Debug)]
pub struct Encoder<'a> {
    pub static_table: StaticTable<'a>,
    pub dynamic_table: DynamicTable,
    pub huffman: Huffman,
}
impl<'a> Encoder<'a> {
    pub fn new(table_size: usize) -> Self {
        Self::with_static_table(table_size, StaticTable::Borrow(STATIC_TABLE))
    }
    pub fn with_static_table(table_size: usize, static_table: StaticTable<'a>) -> Self {
        Self { 
            static_table,
            dynamic_table: DynamicTable::new(table_size),
            huffman: Huffman::new(),
        }
    }

    pub fn find(&self, name: &[u8]) -> Option<usize> {
        for (i, (nam, _)) in self.static_table.iter().enumerate(){
            if name == nam {
                return Some(1 + i);
            }
        }
        for (i, (nam, _)) in self.dynamic_table.table.iter().enumerate(){
            if name == nam {
                return Some(1 + i + self.static_table.len());
            }
        }

        None
    }
    pub fn find_exact(&self, name: &[u8], value: &[u8]) -> Option<usize> {
        for (i, (nam, val)) in self.static_table.iter().enumerate(){
            if name == nam && val == value {
                return Some(1 + i);
            }
        }
        for (i, (nam, val)) in self.dynamic_table.table.iter().enumerate(){
            if name == nam && val == value {
                return Some(1 + i + self.static_table.len());
            }
        }

        None
    }

    pub fn write_int<W: io::Write>(writ: &mut W, value: usize, prefix: u8, stat_prefix: u8) -> io::Result<()> {
        let max = ((1u16 << prefix as u16) - 1) as u8; 

        if value < max as usize {
            writ.write_all(&[stat_prefix | value as u8])
        }
        else {
            writ.write_all(&[stat_prefix | max as u8])?;
            let mut rem = value - max as usize;

            while rem >= 128 {
                let byte = (rem & 0x7f) | 0x80;
                writ.write_all(&[byte as u8])?;
                rem >>= 7;
            }

            writ.write_all(&[rem as u8])
        }
    }
    pub fn write_string<W: io::Write>(&self, writ: &mut W, value: &[u8], use_huff: Option<bool>) -> io::Result<()> {
        if use_huff == Some(true) {
            let huff = self.huffman.encode(value);
            Self::write_int(writ, huff.len(), 7, 0x80)?;
            writ.write_all(&huff)
        }
        else if use_huff == Some(false) {
            Self::write_int(writ, value.len(), 7, 0x00)?;
            writ.write_all(&value)
        }
        else {
            let huff = self.huffman.encode(value);

            if huff.len() > value.len() {
                Self::write_int(writ, value.len(), 7, 0x00)?;
                writ.write_all(&value)
            }
            else {
                Self::write_int(writ, huff.len(), 7, 0x80)?;
                writ.write_all(&huff)
            }
        }
    }

    pub fn write_indexed<W: io::Write>(&self, writ: &mut W, index: usize) -> io::Result<()> {
        Self::write_int(writ, index, 7, 0x80)
    }
    pub fn write_indexed_name<W: io::Write>(&self, writ: &mut W, index: usize, value: &[u8], use_huff: Option<bool>) -> io::Result<()> {
        Self::write_int(writ, index, 6, 0x40)?;
        self.write_string(writ, value, use_huff)
    }
    pub fn write_new_indexed_name<W: io::Write>(&self, writ: &mut W, name: &[u8], value: &[u8], use_huff: Option<bool>) -> io::Result<()> {
        Self::write_int(writ, 0, 6, 0x40)?;
        self.write_string(writ, name, use_huff)?;
        self.write_string(writ, value, use_huff)
    }
    pub fn write_not_indexed<W: io::Write>(&self, writ: &mut W, index: usize, value: &[u8], use_huff: Option<bool>) -> io::Result<()> {
        Self::write_int(writ, index, 4, 0x00)?;
        self.write_string(writ, value, use_huff)
    }
    pub fn write_new_not_indexed<W: io::Write>(&self, writ: &mut W, name: &[u8], value: &[u8], use_huff: Option<bool>) -> io::Result<()> {
        Self::write_int(writ, 0, 4, 0x00)?;
        self.write_string(writ, name, use_huff)?;
        self.write_string(writ, value, use_huff)
    }
    pub fn write_never_indexed<W: io::Write>(&self, writ: &mut W, index: usize, value: &[u8], use_huff: Option<bool>) -> io::Result<()> {
        Self::write_int(writ, index, 4, 0x10)?;
        self.write_string(writ, value, use_huff)
    }
    pub fn write_new_never_indexed<W: io::Write>(&self, writ: &mut W, name: &[u8], value: &[u8], use_huff: Option<bool>) -> io::Result<()> {
        Self::write_int(writ, 0, 4, 0x10)?;
        self.write_string(writ, name, use_huff)?;
        self.write_string(writ, value, use_huff)
    }
    pub fn write_table_size<W: io::Write>(&self, writ: &mut W, new_size: usize) -> io::Result<()> {
        Self::write_int(writ, new_size, 5, 0x20)
    }
    
    pub fn encode<W: io::Write>(&mut self, writ: &mut W, htype: HeaderType, name: &[u8], value: &[u8], use_huff: Option<bool>) -> io::Result<()> {
        if let Some(index) = self.find_exact(name, value) {
            self.write_indexed(writ, index)
        }
        else if let Some(index) = self.find(name) {
            match htype {
                HeaderType::Lookup => Err(io::Error::new(io::ErrorKind::InvalidData, "invalid type, header not in tables")),
                HeaderType::Indexed => {
                    self.write_indexed_name(writ, index, value, use_huff)?;
                    self.dynamic_table.add((name.to_vec(), value.to_vec()));
                    Ok(())
                },
                HeaderType::NotIndexed => self.write_not_indexed(writ, index, value, use_huff),
                HeaderType::NeverIndexed => self.write_never_indexed(writ, index, value, use_huff),
                HeaderType::TableSizeChange => Err(io::Error::new(io::ErrorKind::InvalidData, "invalid type, not available in this function")),
            }
        }
        else {
            match htype {
                HeaderType::Lookup => Err(io::Error::new(io::ErrorKind::InvalidData, "invalid type, header not in tables")),
                HeaderType::Indexed => {
                    self.write_new_indexed_name(writ, name, value, use_huff)?;
                    self.dynamic_table.add((name.to_vec(), value.to_vec()));
                    Ok(())
                },
                HeaderType::NotIndexed => self.write_new_not_indexed(writ, name, value, use_huff),
                HeaderType::NeverIndexed => self.write_new_never_indexed(writ, name, value, use_huff),
                HeaderType::TableSizeChange => Err(io::Error::new(io::ErrorKind::InvalidData, "invalid type, not available in this function")),
            }
        }
    }
    pub fn encode_all<'b, I: Iterator<Item = (&'b [u8], &'b [u8])>>(&mut self, headers: I) -> io::Result<Vec<u8>> {
        let mut buff = Vec::new();

        for (nam, val) in headers {
            self.encode(&mut buff, HeaderType::NotIndexed, nam, val, None)?;
        }

        Ok(buff)
    }
    pub fn encode_all_indexed<'b, I: Iterator<Item = (&'b [u8], &'b [u8])>>(&mut self, headers: I) -> io::Result<Vec<u8>> {
        let mut buff = Vec::new();
        
        for (nam, val) in headers {
            self.encode(&mut buff, HeaderType::Indexed, nam, val, None)?;
        }

        Ok(buff)
    }
}