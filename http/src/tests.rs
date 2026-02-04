#[cfg(test)]

use crate::{http1::{client::Http1Request, server::Http1Socket}, websocket::core::WebSocketFrame};

#[test]
fn two_is_two(){
    assert!(2 == 2)
}

#[tokio::test]
async fn server_client(){
    let (client, server) = tokio::io::duplex(64 * 1024);

    let mut client = Http1Request::new(client, 8 * 1024);
    let mut server = Http1Socket::new(server, 8 * 1024);

    client.path = "/test".to_owned();
    client.send(b"").await.unwrap();
    server.read_until_complete().await.unwrap();
    
    assert_eq!(client.path, server.client.path);
    assert_eq!(client.method, server.client.method);
    assert_eq!(client.version, server.client.version);

    server.close(b"test").await.unwrap();
    client.read_until_complete().await.unwrap();

    assert_eq!(server.code, client.response.code);
    assert!("test".as_bytes() == &client.response.body);
    assert_eq!(server.status, client.response.status.trim());
}

#[test]
fn num_sizes(){
    let int8: u8 = 0;
    let int16: u16 = 0;
    let int32: u32 = 0;
    let int64: u64 = 0;
    let intptr: usize = 0;

    assert_eq!(int8.to_be_bytes().len(), 1);
    assert_eq!(int16.to_be_bytes().len(), 2);
    assert_eq!(int32.to_be_bytes().len(), 4);
    assert_eq!(int64.to_be_bytes().len(), 8);
    assert_eq!(intptr.to_be_bytes().len(), usize::BITS as usize / 8);
}

#[test]
fn websocket_frame(){
    let frame_buff = vec![
        0x82, 0xff, 
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0d,
        0x12, 0x34, 0x56, 0x78,
        0x5a, 0x51, 0x3a, 0x14, 0x7d, 0x18, 0x76, 0x2f, 0x7d, 0x46, 0x3a, 0x1c, 0x6c,
    ];

    let mut frame = WebSocketFrame::from_owned(frame_buff).unwrap();
    let s = String::from_utf8_lossy(frame.unmask_in_place()).to_string();
    let frame = frame;

    assert_eq!(frame.fin, true);
    assert_eq!(frame.rsv, 0);
    assert_eq!(frame.opcode_byte, 2);
    assert_eq!(frame.masked, true);
    assert_eq!(frame.len, 127);
    assert_eq!(frame.ext_len, 13);
    assert_eq!(s, "Hello, World~");

    // dbg!(frame);
}

#[tokio::test]
async fn websocket(){
    let (client, server) = tokio::io::duplex(64 * 1024);

    let mut client = Http1Request::new(client, 8 * 1024);
    let mut server = Http1Socket::new(server, 8 * 1024);

    let f0 = tokio::spawn(async move {
        client.set_header("Host", "localhost");
        client.path = "/test".to_owned();
        let cws = client.websocket_strict().await.unwrap();

        let mut mask = [0u8; 4];
        rand::fill(&mut mask);
        cws.send_text_masked(&mask, b"bin").await.unwrap();

        let frame = cws.read_frame().await.unwrap();
        assert_eq!(frame.fin, true);
        assert_eq!(frame.rsv, 0);
        assert_eq!(frame.opcode_byte, 8);
        assert_eq!(frame.masked, false);
        assert_eq!(frame.len, 9);
        assert_eq!(frame.ext_len, 0);
        assert_eq!(frame.get_payload(), b"\x03\xe8message");
        cws.send_close(1000, b"message").await.unwrap();
    });


    let _ = server.read_until_complete().await.unwrap();
    assert_eq!(server.client.headers.contains_key("sec-websocket-key"), true);
    
    let sws = server.websocket().await.unwrap();
    let mut frame = sws.read_frame().await.unwrap();
    frame.unmask_in_place();
    assert_eq!(frame.fin, true);
    assert_eq!(frame.rsv, 0);
    assert_eq!(frame.opcode_byte, 1);
    assert_eq!(frame.masked, true);
    assert_eq!(frame.len, 3);
    assert_eq!(frame.ext_len, 0);
    assert_eq!(frame.get_payload(), b"bin");
    sws.send_close(1000, b"message").await.unwrap();

    f0.await.unwrap();
}