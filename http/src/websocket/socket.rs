use tokio::{io::{AsyncWriteExt, ReadHalf, WriteHalf}, sync::Mutex};

use crate::{shared::{ReadStream, Stream, WriteStream}, websocket::core::WebSocketFrame};

// pub const MAGIC: [u8; 36] = *b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
pub const MAGIC: &'static [u8] = b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

pub struct WebSocket<R: ReadStream, W: WriteStream>{
    pub netr: Mutex<R>,
    pub netw: Mutex<W>,
}

impl<S: Stream> WebSocket<ReadHalf<S>, WriteHalf<S>>{
    pub fn new(stream: S) -> Self {
        let (netr, netw) = tokio::io::split(stream);
        Self::with_split(netr, netw)
    }
}
impl<R: ReadStream, W: WriteStream> WebSocket<R, W>{
    pub fn with_split(netr: R, netw: W) -> Self {
        Self { netr: Mutex::new(netr), netw: Mutex::new(netw) }
    }


    pub async fn read_frame(&self) -> std::io::Result<WebSocketFrame> {
        let mut netr = self.netr.lock().await;
        let netr = &mut *netr;
        WebSocketFrame::from_stream(netr).await
    }

    pub async fn send_frame_raw(&self, fin: bool, rsv: u8, opcode: u8, mask: Option<&[u8]>, payload: &[u8]) -> std::io::Result<()> {
        let frame = WebSocketFrame::create(fin, rsv, opcode, mask, payload);
        self.netw.lock().await.write(&frame).await?;
        Ok(())
    }

    pub async fn send_text(&self, text: &[u8]) -> std::io::Result<()> {
        self.send_frame_raw(true, 0, 0, None, text).await?;
        Ok(())
    }
    pub async fn send_text_frag(&self, text: &[u8]) -> std::io::Result<()> {
        self.send_frame_raw(false, 0, 0, None, text).await?;
        Ok(())
    }
    pub async fn send_text_masked(&self, mask: &[u8], text: &[u8]) -> std::io::Result<()> {
        self.send_frame_raw(true, 0, 0, Some(mask), text).await?;
        Ok(())
    }
    pub async fn send_text_masked_frag(&self, mask: &[u8], text: &[u8]) -> std::io::Result<()> {
        self.send_frame_raw(false, 0, 0, Some(mask), text).await?;
        Ok(())
    }

    pub async fn send_binary(&self, bin: &[u8]) -> std::io::Result<()> {
        self.send_frame_raw(true, 0, 1, None, bin).await?;
        Ok(())
    }
    pub async fn send_binary_frag(&self, bin: &[u8]) -> std::io::Result<()> {
        self.send_frame_raw(false, 0, 1, None, bin).await?;
        Ok(())
    }
    pub async fn send_binary_masked(&self, mask: &[u8], bin: &[u8]) -> std::io::Result<()> {
        self.send_frame_raw(true, 0, 1, Some(mask), bin).await?;
        Ok(())
    }
    pub async fn send_binary_masked_frag(&self, mask: &[u8], bin: &[u8]) -> std::io::Result<()> {
        self.send_frame_raw(false, 0, 1, Some(mask), bin).await?;
        Ok(())
    }

    // rfc6455 5.5 "All control frames MUST have a payload length of 125 bytes or less and MUST NOT be fragmented."
    pub async fn send_close(&self, code: u16, message: &[u8]) -> std::io::Result<()> {
        if message.len() > 125 { return Err(std::io::Error::new(std::io::ErrorKind::Other, "payload too big")) }
        
        let mut pay = Vec::with_capacity(2 + message.len());
        pay.extend_from_slice(&code.to_be_bytes());
        pay.extend_from_slice(message);

        self.send_frame_raw(true, 0, 8, None, &pay).await?;
        Ok(())
    }
    pub async fn send_close_masked(&self, mask: &[u8], code: u16, message: &[u8]) -> std::io::Result<()> {
        if message.len() > 125 { return Err(std::io::Error::new(std::io::ErrorKind::Other, "payload too big")) }

        let mut pay = Vec::with_capacity(2 + message.len());
        pay.extend_from_slice(&code.to_be_bytes());
        pay.extend_from_slice(message);

        self.send_frame_raw(true, 0, 8, Some(mask), &pay).await?;
        Ok(())
    }

    pub async fn send_ping(&self, payload: &[u8]) -> std::io::Result<()> {
        if payload.len() > 125 { return Err(std::io::Error::new(std::io::ErrorKind::Other, "payload too big")) }
        self.send_frame_raw(true, 0, 9, None, payload).await?;
        Ok(())
    }
    pub async fn send_ping_masked(&self, mask: &[u8], payload: &[u8]) -> std::io::Result<()> {
        if payload.len() > 125 { return Err(std::io::Error::new(std::io::ErrorKind::Other, "payload too big")) }
        self.send_frame_raw(true, 0, 9, Some(mask), payload).await?;
        Ok(())
    }
    pub async fn send_pong(&self, payload: &[u8]) -> std::io::Result<()> {
        if payload.len() > 125 { return Err(std::io::Error::new(std::io::ErrorKind::Other, "payload too big")) }
        self.send_frame_raw(true, 0, 10, None, payload).await?;
        Ok(())
    }
    pub async fn send_pong_masked(&self, mask: &[u8], payload: &[u8]) -> std::io::Result<()> {
        if payload.len() > 125 { return Err(std::io::Error::new(std::io::ErrorKind::Other, "payload too big")) }
        self.send_frame_raw(true, 0, 10, Some(mask), payload).await?;
        Ok(())
    }

}
