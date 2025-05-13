use std::{fs::File, io::Read};

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
        mut dst: &mut tokio_util::bytes::BytesMut,
    ) -> Result<(), Self::Error> {
        let content_len;
        dst.extend_from_slice(
            format!("{:?} {} {}\r\n", item.version(), item.status(), "ok").as_bytes(),
            // format!("{} {} {}\r\n", "HTTP/1.1", "200", "ok").as_bytes(),
        );
        match item.body() {
            ResponseBody::Text(arr) => {
                content_len = arr.len();
            }
            ResponseBody::Json(arr) => {
                dst.extend_from_slice("Content-Type: application/json\r\n".as_bytes());
                content_len = arr.len();
            }
            ResponseBody::File(path) => {
                let f = File::open(path).unwrap();
                content_len = f.metadata().unwrap().len() as usize;
            }
            _ => {
                unimplemented!()
            }
        }

        dst.extend_from_slice(format!("Content-Length: {}\r\n", content_len).as_bytes());

        item.headers().iter().for_each(|(name, value)| {
            dst.extend_from_slice(
                format!(
                    "{}: {}\r\n",
                    name,
                    value.to_str().expect("wrong with headerValue to str!")
                )
                .as_bytes(),
            );
        });
        let e = std::str::from_utf8(&dst).unwrap();
        info!("send half {:?}", e);
        dst.extend_from_slice(b"\r\n");
        match item.body() {
            ResponseBody::Text(arr) => {
                dst.extend_from_slice(&arr.as_slice());
            }
            ResponseBody::Json(arr) => {
                dst.extend_from_slice(&arr.as_slice());
            }
            ResponseBody::File(path) => {
                let mut f = File::open(path).unwrap();
                let mut buf = [0; 1024];
                loop {
                    let r = f.read(&mut buf).unwrap();
                    if r == 0 {
                        break;
                    }
                    dst.extend_from_slice(&buf[0..r]);
                }
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
