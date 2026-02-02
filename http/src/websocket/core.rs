use std::ops::Range;

use tokio::io::AsyncReadExt;

use crate::shared::ReadStream;


pub struct WebSocketFrame{
    pub source: Vec<u8>,
    pub fin: bool,
    pub rsv: u8,
    pub opcode: u8,
    pub masked: bool,
    pub len: u8,
    pub ext_len: u64,
    pub mask: Range<usize>,
    pub payload: Range<usize>,
}

impl WebSocketFrame{
    pub fn from_owned(source: Vec<u8>) -> Option<Self> {
        // let source = buf.into();
        let mut index = 0;

        let fin = source.get(index)? & 0x80 != 0;
        let rsv = (source.get(index)? & 0x70) >> 4;
        let opcode = source.get(index)? & 0xf;
        index += 1;

        let masked = source.get(index)? & 0x80 != 0;
        let len = source.get(index)? & 0x7f;
        index += 1;

        let ext_len =
        if len == 126 {
            index += 2;
            (*source.get(index - 2)? as u64) << 8 | (*source.get(index - 1)? as u64)
        }
        else if len == 127 {
            index += 8;
            (*source.get(index - 8)? as u64) << 56 | (*source.get(index - 7)? as u64) << 48 | (*source.get(index - 6)? as u64) << 40 | (*source.get(index - 5)? as u64) << 32 |
            (*source.get(index - 4)? as u64) << 24 | (*source.get(index - 3)? as u64) << 16 | (*source.get(index - 2)? as u64) << 8 | (*source.get(index - 1)? as u64)
        }
        else {
            0
        };

        let mask = 
        if masked {
            index += 4;
            (index - 4) .. index
        }
        else {
            0..0
        };
        let payload = // index .. source.len();
        if len > 125 {
            index .. (index + ext_len as usize)
        }
        else {
            index .. (index + len as usize)
        };

        Some(Self { 
            source,
            fin,
            rsv,
            opcode,
            masked,
            len,
            ext_len,
            mask,
            payload,
        })
    } 

    pub async fn from_stream<R: ReadStream>(stream: &mut R) -> std::io::Result<Self> {
        let mut source = vec![0u8; 2];
        let mut index = 0;

        stream.read_exact(&mut source).await?;

        let fin = source[index] & 0x80 != 0;
        let rsv = source[index] & 0x70 >> 4;
        let opcode = source[index] & 0xf;
        index += 1;

        let masked = source[index] & 0x80 != 0;
        let len = source[index] & 0x7f;
        index += 1;

        let ext_len =
        if len == 126 {
            index += 2;
            let int = stream.read_u16().await?;
            source.extend_from_slice(&int.to_be_bytes());
            int as u64
        }
        else if len == 127 {
            index += 8;
            let int = stream.read_u64().await?;
            source.extend_from_slice(&int.to_be_bytes());
            int
        }
        else {
            0
        };

        let mask = 
        if masked {
            index += 4;
            source.resize(source.len() + 4, 0);
            stream.read_exact(&mut source[(index - 4)..]).await?;
            (index - 4) .. index
        }
        else {
            0..0
        };

        let payload =
        if len > 125 {
            source.resize(source.len() + ext_len as usize, 0);
            stream.read_exact(&mut source[index..]).await?;
            index .. source.len()
        }
        else {
            source.resize(source.len() + len as usize, 0);
            stream.read_exact(&mut source[index..]).await?;
            index .. source.len()
        };

        Ok(Self {
            source,
            fin,
            rsv,
            opcode,
            masked,
            len,
            ext_len,
            mask,
            payload,
        })
    }

    pub fn create(fin: bool, rsv: u8, opcode: u8, mask: Option<&[u8]>, payload: &[u8]) -> Vec<u8> {

        let mask = if let Some(mask) = mask && mask.len() == 4 { Some(mask) } else { None };

        let length = 2 +
        if mask.is_some() {
            4
        } else {
            0
        } +
        if payload.len() > 0xffff {
            8
        } else if payload.len() > 0x7d {
            2
        } else {
            0
        } +
        payload.len();

        let mut buff = Vec::with_capacity(length);
        let mut pos = 0;
        buff.resize(length, 0);
        
        buff[pos] =
        if fin {
            0x80
        } else {
            0
        } | ((rsv & 0x07) << 4) | (opcode & 0xf);
        pos += 1;

        buff[pos] = 
        if mask.is_some() {
            0x80
        } else {
            0
        } |
        if payload.len() > 0xffff {
            127
        } else if payload.len() > 0x7d {
            126
        } else {
            payload.len() as u8
        };
        pos += 1;

        if payload.len() > 0xffff {
            buff[pos..pos + 8].copy_from_slice(&(payload.len() as u64).to_be_bytes());
            pos += 8;
        } 
        else if payload.len() > 0x7d {
            buff[pos..pos + 2].copy_from_slice(&(payload.len() as u16).to_be_bytes());
            pos += 2;
        }

        if let Some(mask) = mask{
            buff[pos..pos + 4].copy_from_slice(mask);
            pos += 4;
        }

        buff[pos..].copy_from_slice(payload);

        if let Some(mask) = mask{
            let mut m = 0;
            for i in pos..buff.len() {
                buff[i] ^= mask[m];
                m = (m + 1) & 3;
            }
        }

        
        buff
    }


    pub fn unmask_in_place(&mut self) -> &[u8] {
        if self.masked && self.mask.len() == 4 {
            let mask_start = self.mask.start;
            let mut m = 0;
            for i in self.payload.clone() {
                self.source[i] ^= self.source[mask_start + m];
                m = (m + 1) & 3;
            }
        }

        &self.source[self.payload.clone()]
    }

    pub fn get_unmasked(&self) -> Vec<u8> {
        let mut payload = self.source[self.payload.clone()].to_vec();
        
        if self.masked && self.mask.len() == 4 {
            let mask_start = self.mask.start;
            let mut m = 0;
            for i in 0..payload.len() {
                payload[i] ^= self.source[mask_start + m];
                m = (m + 1) & 3;
            }
        }
        
        payload
    }
}