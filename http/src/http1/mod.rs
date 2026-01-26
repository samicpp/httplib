pub mod server;
pub mod client;

fn get_chunk(buf: &[u8]) -> Vec<u8>{
    let mut v = Vec::new();
    let hex = format!("{:X}",buf.len());

    v.extend_from_slice(hex.as_bytes());
    v.extend_from_slice(b"\r\n");
    v.extend_from_slice(buf);
    v.extend_from_slice(b"\r\n");

    v
}
