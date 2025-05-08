use futures::{io, AsyncWrite, StreamExt};
use http::Response;
use tokio::io::AsyncRead;
use tokio_util::codec::Encoder;

pub struct ResponseEncoder {}
impl ResponseEncoder {
    pub fn new() -> Self {
        Self {}
    }
}
impl <T:AsyncWrite> Encoder<Response<T>> for ResponseEncoder {
    type Error = io::Error;
    fn encode(
        &mut self,
        item: Response<T>,
        dst: &mut tokio_util::bytes::BytesMut,
    ) -> Result<(), Self::Error> {
        dst.extend_from_slice(
            format!("{} {} {}\r\n", "HTTP/1.1", item.status().as_str(), "e").as_bytes(),
        );
        item.headers().iter().for_each(|e| {
            dst.extend_from_slice(format!("{}: {}\r\n", e.0, e.1.to_str().unwrap()).as_bytes());
        });
        dst.extend_from_slice(b"\r\n");
        
        Ok(())
    }
}
