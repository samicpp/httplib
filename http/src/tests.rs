#[cfg(test)]

use crate::http1::{client::Http1Request, server::Http1Socket};

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