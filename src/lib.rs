use std::{cell::RefCell, net::TcpStream, ops::DerefMut, rc::Rc};

use request::HttpRequest;
use response::HttpResponse;
use serde::{ Deserialize, Serialize};
mod request;
mod response;
pub fn handle_stream(stream: TcpStream) {
    let rc = Rc::new(RefCell::new(stream));
    let request = HttpRequest::build(rc.clone());
    let response = HttpResponse::init(rc.clone(),request.version());
}

