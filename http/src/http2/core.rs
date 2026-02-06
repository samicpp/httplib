use std::ops::Range;

use tokio::io::AsyncReadExt;

use crate::shared::ReadStream;


#[derive(Debug, Clone)]
pub struct Http2Frame{
    pub source: Vec<u8>,

    pub length: u32,
    pub type_byte: u8,
    pub flags: u8,
    pub stream_id: u32,
    pub pad_len: u8,

    pub priority: Range<usize>,
    pub payload: Range<usize>,
    pub padding: Range<usize>,

    pub ftype: Http2FrameType,
}
impl Http2Frame{
    pub fn from_owned(source: Vec<u8>) -> Option<Self> {
        let length = (source.get(0)? << 16) as u32 | (source.get(1)? << 8) as u32 | *source.get(2)? as u32;
        let type_byte = *source.get(4)?;
        let flags = *source.get(5)?;
        let stream_id = (source.get(6)? << 24) as u32 | (source.get(7)? << 16) as u32 | (source.get(8)? << 8) as u32 | *source.get(9)? as u32;
        let ftype = flags.into();

        let mut pad_len = 0;
        let mut pay_start = 9;
        let mut pay_end = length as usize + 9;

        if flags & 0x08 != 0 {
            pad_len = *source.get(pay_start)?;
            pay_start += 1;
            pay_end -= pad_len as usize;
        }
        else if flags & 0x20 != 0 {
            pay_start += 5;
        }

        let priority = 9..pay_start;
        let payload = pay_start..pay_end;
        let padding = pay_end..length as usize;

        Some(Self {
            source,
            length,
            type_byte,
            flags,
            stream_id,
            pad_len,

            priority,
            payload,
            padding,

            ftype,
        })
    }
    pub async fn from_reader<R: ReadStream>(stream: &mut R) -> Result<Self, std::io::Error> {
        let mut source = vec![0; 9];
        stream.read_exact(&mut source).await?;

        let length = (source[0] << 16) as u32 | (source[1] << 8) as u32 | source[2] as u32;
        let type_byte = source[4];
        let flags = source[5];
        let stream_id = (source[6] << 24) as u32 | (source[7] << 16) as u32 | (source[8] << 8) as u32 | source[9] as u32;
        let ftype = flags.into();

        let mut pad_len = 0;
        let mut pay_start = 9;
        let mut pay_end = length as usize + 9;

        if flags & 0x08 != 0 {
            pad_len = source[pay_start];
            pay_start += 1;
            pay_end -= pad_len as usize;
        }
        else if flags & 0x20 != 0 {
            pay_start += 5;
        }

        let priority = 9..pay_start;
        let payload = pay_start..pay_end;
        let padding = pay_end..length as usize;

        source.resize(9 + length as usize, 0);
        stream.read_exact(&mut source[9..]).await?;

        Ok(Self {
            source,
            length,
            type_byte,
            flags,
            stream_id,
            pad_len,

            priority,
            payload,
            padding,

            ftype,
        })
    }

    pub fn create(ftype: Http2FrameType, flags: u8, stream_id: u32, priority: Option<&[u8]>, payload: Option<&[u8]>, padding: Option<&[u8]>) -> Vec<u8> {
        let mut priority = priority.filter(|s| s.len() == 5);
        let mut payload = payload.filter(|s| s.len() <= 16777216);
        let mut padding = padding.filter(|s| s.len() <= 256);

        let length = 
            priority.and_then(|s| Some(s.len())).unwrap_or(0) + 
            payload.and_then(|s| Some(s.len())).unwrap_or(0) + 
            padding.and_then(|s| Some(s.len())).unwrap_or(0);

        let length =         
        if length > 16777216 {
            priority = None;
            payload = None;
            padding = None;
            0
        }
        else {
            length
        };
        
        let mut frame = vec![0; 9 + length];

        frame[0] = ((length & 0xff0000) >> 16) as u8;
        frame[1] = ((length & 0x00ff00) >> 8) as u8;
        frame[2] = length as u8;

        frame[3] = ftype.into();
        frame[4] = flags |
        if priority.is_some() { 0x20 } else { 0x00 } |
        if padding. is_some() { 0x08 } else { 0x00 } ;

        frame[5..9].copy_from_slice(&u32::to_be_bytes(stream_id));

        let mut start = 9;

        if let Some(pad) = padding {
            frame[start] = pad.len() as u8;
            start += 1;
        }
        if let Some(priority) = priority {
            frame[start..start + 4].copy_from_slice(priority);
            start += 4;
        }
        if let Some(payload) = payload {
            frame[start..start + payload.len()].copy_from_slice(payload);
        }
        if let Some(padding) = padding {
            let off = frame.len() - padding.len();
            frame[off..].copy_from_slice(padding);
        }

        frame
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Http2FrameType{
    Data,
    Headers,
    Priority,
    RstStream,
    Settings,
    PushPromise,
    Ping,
    Goaway,
    WindowUpdate,
    Continuation,
    
    Invalid(u8),
}
impl From<u8> for Http2FrameType {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Data,
            1 => Self::Headers,
            2 => Self::Priority,
            3 => Self::RstStream,
            4 => Self::Settings,
            5 => Self::PushPromise,
            6 => Self::Ping,
            7 => Self::Goaway,
            8 => Self::WindowUpdate,
            9 => Self::Continuation,

            v => Self::Invalid(v),
        }
    }
}
impl Into<u8> for Http2FrameType {
    fn into(self) -> u8 {
        match self {
            Self::Data => 0,
            Self::Headers => 1,
            Self::Priority => 2,
            Self::RstStream => 3,
            Self::Settings => 4,
            Self::PushPromise => 5,
            Self::Ping => 6,
            Self::Goaway => 7,
            Self::WindowUpdate => 8,
            Self::Continuation => 9,

            Self::Invalid(v) => v,
        }
    }
}