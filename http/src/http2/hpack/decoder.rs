use crate::http2::hpack::{DynamicTable, HeaderType, STATIC_TABLE, StaticTable, huffman::Huffman};


pub struct Decoder<'a> {
    pub static_table: StaticTable<'a>,
    pub dynamic_table: DynamicTable,
    pub huffman: Huffman,
}
impl<'a> Decoder<'a> {
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

    pub fn get(&self, index: usize) -> Option<(&[u8], &[u8])> {
        let ssize = self.static_table.len();
        let dsize = self.dynamic_table.table.len();

        if 1 <= index && index <= ssize {
            self.static_table.get(index - 1)
        }
        else if ssize < index && index <= ssize + dsize {
            self.dynamic_table.table.get(index - 1 - ssize).map(|(h, v)| (h.as_slice(), v.as_slice()))
        }
        else {
            None
        }
    }

    pub fn read_int(buf: &[u8], prefix: u8, pos: &mut usize) -> Option<usize> {
        // if prefix == 0 || prefix > 8 { return None; }

        let mask = ((1u16 << prefix as u16) - 1) as u8;
        let mut value = (buf.get(*pos)? & mask) as usize;
        *pos += 1;

        if value < mask as usize { return Some(value) }

        let mut m = 0;

        for &b in buf[*pos..].iter() {
            *pos += 1;
            value += ((b & 0x7f) as usize) * (1 << m);
            m += 7;

            if b & 0x80 == 0 {
                return Some(value);
            }
        }

        None
    }
    pub fn read_string(&self, buf: &[u8], pos: &mut usize) -> Option<Vec<u8>> {
        let huff = (*buf.get(*pos)? & 0x80) != 0;
        let length = Self::read_int(buf, 7, pos)?;

        if *pos + length > buf.len() {
            None
        }
        else {
            let buff = &buf[*pos..*pos + length];
            *pos += length;
            
            if huff { self.huffman.decode(buff).ok() }
            else { Some(buff.to_vec()) }
        }
    }

    pub fn decode(&mut self, buf: &[u8], pos: &mut usize) -> Option<(HeaderType, Vec<u8>, Vec<u8>)> {
        let first = *buf.get(*pos)?;
        let mut output = None;

        if first & 0x80 != 0 {
            // 6.1 Indexed Header Field
            let index = Self::read_int(buf, 7, pos)?;

            let fromt = self.get(index)?;
            let name = fromt.0.to_vec();
            let valu = fromt.1.to_vec();

            output = Some((HeaderType::Lookup, name, valu));
        }
        // 6.2 Literal Header Field Representation
        else if first & 0xc0 == 0x40 {
            // 6.2.1 Literal Header Field with Incremental Indexing
            let index = Self::read_int(buf, 6, pos)?;

            let name =
            if index == 0 { self.read_string(buf, pos)? }
            else { self.get(index)?.0.to_vec() };

            let valu = self.read_string(buf, pos)?;

            self.dynamic_table.add((name.clone(), valu.clone()));
            output = Some((HeaderType::Indexed, name, valu));
        }
        else if first & 0xf0 == 0x00 {
            // 6.2.2 Literal Header Field without Indexing
            let index = Self::read_int(buf, 4, pos)?;

            let name =
            if index == 0 { self.read_string(buf, pos)? }
            else { self.get(index)?.0.to_vec() };

            let valu = self.read_string(buf, pos)?;

            output = Some((HeaderType::NotIndexed, name, valu));
        }
        else if first & 0xf0 == 0x10 {
            // 6.2.3 Literal Header Field Never Indexed
            let index = Self::read_int(buf, 4, pos)?;

            let name =
            if index == 0 { self.read_string(buf, pos)? }
            else { self.get(index)?.0.to_vec() };

            let valu = self.read_string(buf, pos)?;

            output = Some((HeaderType::NeverIndexed, name, valu));
        }
        else if first & 0xe0 == 0x20 {
            // 6.3 Dynamic Table Size Update
            let new_size = Self::read_int(buf, 5, pos)?;
            self.dynamic_table.resize(new_size);
            output = Some((HeaderType::TableSizeChange, vec![], vec![]));
        }
        else {
            // None?
        }
        
        
        output
    }
    pub fn decode_all(&mut self, buf: &[u8]) -> Option<Vec<(Vec<u8>, Vec<u8>)>> {
        let mut dec = Vec::new();
        let mut pos = 0;

        while pos < buf.len() {
            let (t, h, v) = self.decode(buf, &mut pos)?;
            
            if t != HeaderType::TableSizeChange {
                dec.push((h, v))
            }
        }
        
        Some(dec)
    }
}
