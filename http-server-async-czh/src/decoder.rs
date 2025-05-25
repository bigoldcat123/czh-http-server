use std::{io, str::FromStr};

use http::{HeaderName, HeaderValue, Request};
use log::info;
use tokio_util::{bytes::Buf, codec::Decoder};

use crate::body_type::RequestBody;

pub struct RequestDecoder {}
impl RequestDecoder {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl Decoder for RequestDecoder {
    type Error = io::Error;
    type Item = Request<RequestBody>;
    fn decode(
        &mut self,
        src: &mut tokio_util::bytes::BytesMut,
    ) -> Result<Option<Self::Item>, Self::Error> {
        let mut iter = src.iter().as_slice().split(|x| *x == b'\n');
        let mut header_line_len = 0;
        let mut body_len = 0;
        let head_line = match iter.next() {
            Some(h) => {
                if h.contains(&b'\r') {
                    header_line_len += h.len() + 1;
                    match std::str::from_utf8(h) {
                        Ok(r) => r,
                        Err(e) => {
                            return Err(io::Error::new(io::ErrorKind::InvalidData, e.to_string()));
                        }
                    }
                } else {
                    return Ok(None);
                }
            }
            None => return Ok(None),
        };
        let head_line = head_line.split_whitespace().collect::<Vec<&str>>();
        if head_line.len() != 3 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid Data"));
        }
        let mut req = Request::builder().method(head_line[0]).uri(head_line[1]);
        let headers = req.headers_mut().unwrap();
        info!("{:?}", head_line);
        for line in iter {
            header_line_len += line.len() + 1;

            if line.len() == 1 {
                break;
            }

            let l_str = match std::str::from_utf8(line) {
                Ok(s) => s,
                Err(e) => {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, e.to_string()));
                }
            };
            info!("{:?}", l_str);
            let (k, v) = match l_str.trim().split_once(":") {
                Some(e) => e,
                None => {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid header"));
                }
            };
            headers.insert(
                HeaderName::from_str(k).unwrap(),
                HeaderValue::from_str(v).unwrap(),
            );
            if k.eq_ignore_ascii_case("content-length") {
                body_len = match v.trim().parse::<usize>() {
                    Ok(e) => e,
                    Err(e) => {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, e.to_string()));
                    }
                }
            }
        }
        info!("{:?} {}", header_line_len, body_len);
        if header_line_len + body_len <= src.len() {
            src.advance(header_line_len + body_len);
            
            Ok(Some(
                req.body(RequestBody::Text("body".to_string())).unwrap(),
            ))
        } else {
            Ok(None)
        }
    }
}
