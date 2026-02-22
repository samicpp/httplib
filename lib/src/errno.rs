use http::shared::LibError;

pub const NO_ERR: i32 = -1;
pub const TYPE_ERR: i32 = 1;
pub const ERROR: i32 = 0x100;
pub const IO_ERROR: i32 = 0x200;


pub fn errno_liberr(error: &LibError) -> i32 {
    match error {
        LibError::Io(io) => io.raw_os_error().unwrap_or(0) | IO_ERROR,
        LibError::Huffman(_) => 0x101,
        LibError::Hpack(_) => 0x102,

        LibError::NotConnected => 0x102,
        LibError::ConnectionClosed => 0x103,
        LibError::StreamClosed => 0x104,
        LibError::HeadersSent => 0x105,

        LibError::Invalid => 0x106,
        LibError::InvalidFrame => 0x107,
        LibError::InvalidUpgrade => 0x108,
        LibError::InvalidStream => 0x109,
        LibError::InvalidString => 0x110,

        LibError::NotAccepted => 0x111,
        LibError::ResetStream => 0x112,
        LibError::Goaway => 0x113,
        LibError::ProtocolError => 0x114,
    }
}

pub trait Errno {
    fn get_errno(&self) -> i32;
}

impl Errno for LibError {
    fn get_errno(&self) -> i32 {
        match self {
            Self::Io(io) => io.get_errno(),
            Self::Huffman(_) => 0x101,
            Self::Hpack(_) => 0x102,

            Self::NotConnected => 0x102,
            Self::ConnectionClosed => 0x103,
            Self::StreamClosed => 0x104,
            Self::HeadersSent => 0x105,

            Self::Invalid => 0x106,
            Self::InvalidFrame => 0x107,
            Self::InvalidUpgrade => 0x108,
            Self::InvalidStream => 0x109,
            Self::InvalidString => 0x110,

            Self::NotAccepted => 0x111,
            Self::ResetStream => 0x112,
            Self::Goaway => 0x113,
            Self::ProtocolError => 0x114,
        }
    }
}
impl Errno for std::io::Error {
    fn get_errno(&self) -> i32 {
        self.raw_os_error().unwrap_or(0) | IO_ERROR
    }
}
