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
    
    // let data = MockStudent {
    //     name: String::from("zhangsan"),
    //     age: 18,
    //     maybeNone: None,
    //     boxValue:Box::new(1),
    // };
    // response.json(data);
    // println!("{:#?}", request)
}

// #[derive(Serialize, Deserialize)]
// struct MockStudent {
//     name: String,
//     age: u8,
//     maybeNone:Option<u8>,
//     boxValue:Box<u8>,
// }