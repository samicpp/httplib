pub mod hpack;
pub mod core;
pub mod session;
pub mod client;
pub mod server;

pub const PREFACE: &'static [u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";
