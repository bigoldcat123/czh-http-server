use std::io;

use futures::future::ok;
use http::{Method, Request};
use log::info;
use tokio_util::{bytes::Buf, codec::Decoder};

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
        info!("here!");
        let mut iter = src.iter().as_slice().split(|x| *x == b'\n');
        let mut header_line_len = 0;
        let mut body_len = 0;
        let head_line = match iter.next() {
            Some(h) => {
                if h.contains(&b'\r') {
                    h
                } else {
                    return Ok(None);
                }
            }
            None => return Ok(None),
        };
        info!("{:?}", std::str::from_utf8(head_line).unwrap());
        header_line_len += head_line.len() + 1;
        for line in iter {
            header_line_len += line.len() + 1;
            let l_str = match std::str::from_utf8(line) {
                Ok(s) => s,
                Err(e) => {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, e.to_string()));
                }
            };
            info!("{:?}", l_str);
            let l = l_str.trim().split(":").collect::<Vec<&str>>();
            if l.len() != 2 {
                info!("{:?}",l);
                return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid header"));
            }

            if l[0].eq_ignore_ascii_case("content-type") {
                body_len = match l[1].parse::<usize>() {
                    Ok(e) => e,
                    Err(e) => {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, e.to_string()));
                    }
                }
            }
            if line.len() == 1 {
                break;
            }
        }
        info!("{:?} {}", header_line_len, body_len,);
        if header_line_len + body_len <= src.len() {
            src.advance(header_line_len + body_len);
            Ok(Some(Request::new("body".to_string())))
        } else {
            Ok(None)
        }
    }
}
