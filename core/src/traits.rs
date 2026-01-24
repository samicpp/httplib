use tokio::io::{AsyncRead, AsyncWrite};



pub trait Stream: AsyncRead + AsyncWrite + Unpin + Send + Sync {}
impl<A> Stream for A where A: AsyncRead + AsyncWrite + Unpin + Send + Sync {}

