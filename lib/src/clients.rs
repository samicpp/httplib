use std::{io, sync::Arc};
use tokio_rustls::{rustls, TlsConnector, rustls::ClientConfig};
use tokio::net::TcpStream;


use crate::DynStream;


pub async fn tcp_connect(addr: String) -> io::Result<DynStream> {
    // not alot going on
    let stream: TcpStream = TcpStream::connect(addr).await?;
    Ok(DynStream::Tcp(stream))
}

pub async fn tls_connect(addr: String, domain: String) -> io::Result<DynStream> {
    let root = rustls::RootCertStore::empty();
    let conf = ClientConfig::builder().with_root_certificates(root).with_no_client_auth();
    let conn = TlsConnector::from(Arc::new(conf));
    let tcp: TcpStream = TcpStream::connect(addr).await?;
    let domain = rustls::pki_types::ServerName::try_from(domain);
    
    match domain {
        Ok(domain) => {
            let tls = conn.connect(domain, tcp).await?;
            Ok(tls.into())
        },
        Err(_) => Err(io::Error::new(io::ErrorKind::Other, "invalid domain")),
    }
}
