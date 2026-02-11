use std::{cmp::min, io, sync::{Arc, Mutex as SyncMutex, atomic::{AtomicBool, AtomicU32}}};

use dashmap::{DashMap, mapref::one::RefMut};
use tokio::{io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf}, sync::{Mutex as AsyncMutex, Notify}};

use crate::{http2::{core::{Http2Frame, Http2FrameType, Http2Settings}, hpack::{HeaderType, decoder::Decoder, encoder::Encoder}}, shared::{LibError, LibResult, ReadStream, Stream, WriteStream}};

pub const PREFACE: &'static [u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";


#[derive(Debug)]
pub struct Http2Data {
    pub window: usize,
    pub notify: Arc<Notify>,

    pub stream_id: u32,
    pub reset: bool,

    pub end_head: bool,
    pub end_body: bool,
    pub self_end_head: bool,
    pub self_end_body: bool,
    
    pub body: Vec<u8>,
    pub head: Vec<u8>,
    pub headers: Vec<(Vec<u8>, Vec<u8>)>,
}
impl Http2Data {
    pub fn empty(stream_id: u32, sett: Http2Settings) -> Self {
        Self {
            window: sett.initial_window_size.unwrap_or(65535) as usize,
            notify: Arc::new(Notify::new()),
            stream_id,
            reset: false,
            end_head: false,
            end_body: false,
            self_end_head: false,
            self_end_body: false,
            body: Vec::new(),
            head: Vec::new(),
            headers: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct Http2Session<R: ReadStream, W: WriteStream>{
    pub netr: AsyncMutex<R>,
    pub netw: AsyncMutex<W>,

    pub decoder: AsyncMutex<Decoder<'static>>,
    pub encoder: AsyncMutex<Encoder<'static>>,

    pub max_stream_id: AtomicU32,
    pub streams: DashMap<u32, Http2Data>,

    pub goaway: AtomicBool,
    pub goaway_frame: SyncMutex<Option<Http2Frame>>,

    pub window: SyncMutex<usize>,
    pub notify: Notify,

    pub settings: SyncMutex<Http2Settings>,
}
impl<S: Stream> Http2Session<ReadHalf<S>, WriteHalf<S>> {
    pub fn new(net: S) -> Self {
        let (netr, netw) = tokio::io::split(net);
        Self::with(netr, netw, Http2Settings::default())
    }
}
impl<R: ReadStream, W: WriteStream> Http2Session<R, W> {
    pub fn with(netr: R, netw: W, settings: Http2Settings) -> Self {
        let netr = AsyncMutex::new(netr);
        let netw = AsyncMutex::new(netw);

        Self {
            netr, netw,
            decoder: AsyncMutex::new(Decoder::new(settings.header_table_size.unwrap_or(4096) as usize)),
            encoder: AsyncMutex::new(Encoder::new(settings.header_table_size.unwrap_or(4096) as usize)),
            max_stream_id: AtomicU32::new(0),
            streams: DashMap::new(),
            goaway: AtomicBool::new(false),
            goaway_frame: SyncMutex::new(None),
            window: SyncMutex::new(settings.initial_window_size.unwrap_or(65535) as usize),
            notify: Notify::new(),
            settings: SyncMutex::new(settings)
        }
    }


    pub async fn send_preface(&self) -> io::Result<()> {
        self.netw.lock().await.write_all(PREFACE).await
    }
    pub async fn read_preface(&self) -> io::Result<bool> {
        let mut pre = [0; PREFACE.len()];
        self.netr.lock().await.read_exact(&mut pre).await?;
        Ok(pre == PREFACE)
    }

    pub async fn read_frame(&self) -> io::Result<Http2Frame> {
        let mut reader = self.netr.lock().await;
        Http2Frame::from_reader(&mut *reader).await
    }
    pub async fn next(&self) -> LibResult<u32> {
        let frame = self.read_frame().await?;
        self.handle(frame).await
    }
    pub async fn handle(&self, frame: Http2Frame) -> LibResult<u32> {
        match frame.ftype {
            
            _ => (),
        }
        todo!()
    }

    pub fn get_or_open_stream(&self, stream_id: u32) -> RefMut<'_, u32, Http2Data> {
        if let Some(shard) = self.streams.get_mut(&stream_id) {
            shard
        }
        else {
            self.streams.insert(stream_id, Http2Data::empty(stream_id, *self.settings.lock().unwrap()));
            self.streams.get_mut(&stream_id).unwrap()
        }
    }


    pub async fn write_frame(&self, ftype: Http2FrameType, flags: u8, stream_id: u32, priority: Option<&[u8]>, payload: Option<&[u8]>, padding: Option<&[u8]>) -> io::Result<()> {
        self.netw.lock().await.write_all(&Http2Frame::create(ftype, flags, stream_id, priority, payload, padding)).await
    }

    pub async fn send_data(&self, stream_id: u32, end: bool, buf: &[u8]) -> LibResult<()> {
        // let mut stream = 
        let notify =
        if let Some(mut shard) = self.streams.get_mut(&stream_id) {
            if shard.self_end_body || shard.reset {
                return Err(LibError::StreamClosed)
            }

            shard.self_end_body = end;
            shard.notify.clone()
        }
        else {
            return Err(LibError::InvalidStream)
        };

        if buf.len() == 0 {
            if end {
                self.write_frame(Http2FrameType::Data, 1, stream_id, None, None, None).await?;
            }
            return Ok(());
        }

        let mut pos = 0;
        let mut buff = vec![];
        let mfs = self.settings.lock().unwrap().max_frame_size.unwrap_or(16384) as usize;
        // let mut window = self.window.lock().unwrap();
        // let mut minim = min(mfs, min(*window, stream.window));

        // while buf.len() - pos > minim {
        while buf.len() > pos {
            let (max, ncws, nsws) =
            {
                let mut window = self.window.lock().unwrap();
                let mut stream = self.streams.get_mut(&stream_id).unwrap();

                if stream.reset {
                    return Err(LibError::ResetStream)
                }

                let max = min(buf.len() - pos, min(*window, stream.window));
                *window -= max;
                stream.window -= max;
                // drop(window);
                // drop(stream);
                (max, *window, stream.window)
            };


            if max > 0 {
                let chunks = max / mfs;
                let rem = max % mfs;

                for _ in 0..chunks {
                    buff.append(&mut Http2Frame::create(Http2FrameType::Data, 0, stream_id, None, Some(&buf[pos..pos + mfs]), None));
                    pos += mfs;
                }

                buff.append(&mut Http2Frame::create(Http2FrameType::Data, 0, stream_id, None, Some(&buf[pos..pos + rem]), None));
                pos += rem;

                self.netw.lock().await.write_all(&buff).await?;
                buff.clear();
            }

            if nsws == 0 {
                notify.notified().await;
            }
            else if ncws == 0 {
                self.notify.notified().await;
            }
            else {
                tokio::select! {
                    _ = notify.notified() => {},
                    _ = self.notify.notified() => {},
                }
            }

            // window = self.window.lock().unwrap();
            // stream = self.streams.get_mut(&stream_id).unwrap();
        }

        if end {
            self.write_frame(Http2FrameType::Data, 1, stream_id, None, None, None).await?;
        }

        todo!()
    }

    pub async fn send_headers(&self, stream_id: u32, end: bool, headers: &[(&[u8], &[u8])]) -> LibResult<()> {
        {
            let mut shard = self.get_or_open_stream(stream_id);

            if shard.self_end_head || shard.self_end_body {
                return Err(LibError::StreamClosed)
            }

            shard.self_end_head = true;
            shard.self_end_body = end;
        }

        let mut hpacke = self.encoder.lock().await;
        let enc = {
            let mut buff = Vec::new();

            for &(nam, val) in headers {
                hpacke.encode(&mut buff, HeaderType::NotIndexed, nam, val, None)?;
            }

            buff
        };
        let mfs = self.settings.lock().unwrap().max_frame_size.unwrap_or(16384) as usize;
        
        let mut pos = 0;
        let mut buff = Vec::with_capacity(9 + enc.len() / mfs * 9 + enc.len());

        
        if enc.len() < mfs {
            buff.append(&mut Http2Frame::create(Http2FrameType::Headers, if end { 5 } else { 4 }, stream_id, None, Some(&enc), None));
        }
        else {
            buff.append(&mut Http2Frame::create(Http2FrameType::Headers, 0, stream_id, None, Some(&enc[pos..pos + mfs]), None));
            pos += mfs;

            let mut chunks = enc.len() / mfs;

            if enc.len() % mfs == 0 {
                chunks -= 1;
                // rem += mfs;
            }

            for _ in 0..chunks {
                buff.append(&mut Http2Frame::create(Http2FrameType::Continuation, 0, stream_id, None, Some(&enc[pos..pos + mfs]), None));
                pos += mfs;
            }

            buff.append(&mut Http2Frame::create(Http2FrameType::Continuation, if end { 5 } else { 4 }, stream_id, None, Some(&enc[pos..]), None));
        }

        self.netw.lock().await.write_all(&buff).await?;
        drop(hpacke);

        Ok(())
    }

    // pub async fn send_priority(&self, stream_id: u32) -> io::Result<()> { todo!() }
    
    pub async fn send_rst_stream(&self, stream_id: u32, code: u32) -> io::Result<()> { 
        self.write_frame(Http2FrameType::RstStream, 0, stream_id, None, Some(&u32::to_be_bytes(code)), None).await
    }

    pub async fn send_settings(&self, settings: Http2Settings) -> io::Result<()> { 
        self.write_frame(Http2FrameType::Settings, 0, 0, None, Some(&settings.to_vec()), None).await
    }
    
    // pub async fn send_push_promise(&self, stream_id: u32) -> io::Result<()> { todo!() }
    
    pub async fn send_ping(&self, ack: bool, buf: &[u8]) -> io::Result<()> { 
        self.write_frame(Http2FrameType::Ping, if ack { 1 } else { 0 }, 0, None, Some(buf), None).await
    }
    
    pub async fn send_goaway(&self, stream_id: u32, code: u32, buf: &[u8]) -> io::Result<()> {
        let mut pay = vec![];
        
        pay.extend_from_slice(&u32::to_be_bytes(stream_id));
        pay.extend_from_slice(&u32::to_be_bytes(code));
        pay.extend_from_slice(buf);

        self.write_frame(Http2FrameType::Goaway, 0, 0, None, Some(&pay), None).await
    }
    
    pub async fn send_window_update(&self, stream_id: u32, size: u32) -> io::Result<()> {
        self.write_frame(Http2FrameType::WindowUpdate, 0, stream_id, None, Some(&u32::to_be_bytes(size)), None).await
    }
    
    // pub async fn send_continuation(&self, stream_id: u32) -> io::Result<()> { unimplemented!() } // no reason for this

}