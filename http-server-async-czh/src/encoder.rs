use futures::io;
use http::Response;
use log::info;
use tokio_util::codec::Encoder;

use crate::body_type::ResponseBody;

pub struct ResponseEncoder {}
impl ResponseEncoder {
    pub fn new() -> Self {
        Self {}
    }
}
impl Encoder<Response<ResponseBody>> for ResponseEncoder {
    type Error = io::Error;
    fn encode(
        &mut self,
        item: Response<ResponseBody>,
        dst: &mut tokio_util::bytes::BytesMut,
    ) -> Result<(), Self::Error> {
        dst.extend_from_slice(
            format!("{:?} {} {}\r\n", item.version(), item.status(), "ok").as_bytes(),
        );

        match item.body() {
            ResponseBody::Text(arr) => {
                dst.extend_from_slice(format!("Content-Length: {}\r\n", arr.len()).as_bytes());
            }
            _ => {
                unimplemented!()
            }
        }

        item.headers().iter().for_each(|e| {
            dst.extend_from_slice(format!("{}: {}\r\n", e.0, e.1.to_str().unwrap()).as_bytes());
        });
        // dst.extend_from_slice("Content-Length: 10\r\n".as_bytes());
        dst.extend_from_slice(b"\r\n");
        match item.body() {
            ResponseBody::Text(arr) => {
                dst.extend_from_slice(&arr.as_slice());
            }
            _ => {
                unimplemented!()
            }
        }
        let e = std::str::from_utf8(&dst).unwrap();
        info!("send ok {:?}", e);
        Ok(())
    }
}
