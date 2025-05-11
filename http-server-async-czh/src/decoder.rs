use std::io;

use http::{Method, Request};
use tokio_util::codec::Decoder;

pub struct RequestDecoder {}
impl RequestDecoder {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl Decoder for RequestDecoder {
    type Error = io::Error;
    type Item = Request<String>;
    fn decode(
        &mut self,
        src: &mut tokio_util::bytes::BytesMut,
    ) -> Result<Option<Self::Item>, Self::Error> {
        src.clear();
        let req = Request::builder()
            .uri("https://localhost:8846")
            .method(Method::POST)
            .body("body".to_string())
            .unwrap();
        Ok(Some(req))
    }
}
