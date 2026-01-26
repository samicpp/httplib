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