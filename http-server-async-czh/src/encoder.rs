use futures::io;
use http::Response;
use log::info;
use tokio_util::codec::Encoder;

pub struct ResponseEncoder {}
impl ResponseEncoder {
    pub fn new() -> Self {
        Self {}
    }
}
impl Encoder<Response<Vec<u8>>> for ResponseEncoder {
    type Error = io::Error;
    fn encode(
        &mut self,
        item: Response<Vec<u8>>,
        dst: &mut tokio_util::bytes::BytesMut,
    ) -> Result<(), Self::Error> {
        dst.extend_from_slice(
            format!("{:?} {} {}\r\n", item.version(), item.status(), "ok").as_bytes(),
        );
        item.headers().iter().for_each(|e| {
            dst.extend_from_slice(format!("{}: {}\r\n", e.0, e.1.to_str().unwrap()).as_bytes());
        });
        // dst.extend_from_slice("Content-Length: 10\r\n".as_bytes());
        dst.extend_from_slice(b"\r\n");
        // dst.extend_from_slice("aaaaaaaaaa".as_bytes());
        dst.extend_from_slice(item.body().as_slice());
        let e = std::str::from_utf8(&dst).unwrap();
        info!("send ok {:?}", e);
        Ok(())
    }
}
