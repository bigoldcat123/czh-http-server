//!
//! # czh_http_server
//!
//! czh_http_server is a simple http server
//!
use std::{
    cell::RefCell, collections::HashMap, io::LineWriter, net::{TcpListener, TcpStream}, ops::DerefMut, rc::Rc, sync::Arc
};

use controller::Controller;
use request::HttpRequest;
use response::HttpResponse;
mod request;
mod response;
mod controller;
type ControllerHandler = Box<dyn Fn(HttpRequest,HttpResponse) + Sync + Send + 'static>;

pub struct HttpServer {
    listener: TcpListener,
    port: u16,
    controller:Option<Controller>
}
impl HttpServer {
    pub fn create_server(host: &str, port: u16) -> Self {
        let listener = TcpListener::bind(format!("{}:{}", host, port)).unwrap();
        HttpServer { listener, port ,controller:Some( Controller::new() )}
    }
    pub fn listen(mut self) {
        let controller = Arc::new(self.controller.take().unwrap());
        let pool = czhmt::ThreadPool::new(4);
        for client in self.listener.incoming() {
            match client {
                Ok(stream) => {
                    let controller = controller.clone();
                    pool.exec(|| {
                        handle_stream(stream, controller);
                    });
                }
                Err(_) => {
                    println!("Something wrong!")
                }
            }
        }
    }
}

pub fn handle_stream(stream: TcpStream, controller: Arc<Controller>) {
    let rc = Rc::new(RefCell::new(stream));
    let request = HttpRequest::build(rc.clone());
    let response = HttpResponse::init(rc.clone(), request.version());
    controller.handle_request(request, response);
}
