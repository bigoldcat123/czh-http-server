//!
//! # czh_http_server
//!
//! czh_http_server is a simple http server
//!
use std::{
    cell::RefCell, fs::File, mem, net::{TcpListener, TcpStream}, path::{Path, PathBuf}, rc::Rc, sync::Arc
};

use controller::Controller;
use request::HttpRequest;
use response::{ContentType, HttpResponse};
mod controller;
pub mod request;
pub mod response;
pub mod route;
type ControllerHandler = Box<dyn Fn(HttpRequest, HttpResponse) + Sync + Send + 'static>;
pub trait HttpHander {
    fn get<T>(&mut self, url: &str, controller: T)
    where
        T: Fn(HttpRequest, HttpResponse) + Sync + Send + 'static;

    fn post<T>(&mut self, url: &str, controller: T)
    where
        T: Fn(HttpRequest, HttpResponse) + Sync + Send + 'static;
    fn router(&mut self, url: &str, route: route::Route);
}



pub struct HttpServer {
    listener: TcpListener,
    // port: u16,
    controller: Option<Controller>,
}
impl HttpServer {
    pub fn create_server(host: &str, port: u16) -> Self {
        let listener = TcpListener::bind(format!("{}:{}", host, port)).unwrap();
        HttpServer {
            listener,
            controller: Some(Controller::new()),
        }
    }
    pub fn listen(mut self) {
        let controller = Arc::new(self.controller.take().unwrap());
        let pool = czhmt::ThreadPool::new(4);
        for client in self.listener.incoming() {
            print!("hello");
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

    ///
    /// url prefix
    /// absolute path
    /// say
    /// `
    /// map("/path","/file")
    /// `
    /// the url /path/a would be mapped to /file/a which is a fs path
    pub fn map(&mut self, url: &str, path: &str) {
        let path = path.to_string();
        self.controller
            .as_mut()
            .unwrap()
            .add_static_handler(url, move |req, res| {
                // println!("map {:#?} to ",url,path);
                println!("map {} to {:#?}", req.url(), path);
                let root = PathBuf::from(path.as_str());
                let mut start = 1;
                loop {
                    if start >= req.url().len() {
                        break;
                    }
                    if req.url().as_bytes()[start] == b'/' {
                        start += 1;
                        break;
                    }
                    start += 1;
                }
                let mut path = root.join(req.url()[start..].to_string());
                println!("{:#?}", path);
                if let Ok(file) = File::open(&path) {
                    if file.metadata().unwrap().is_dir() {
                        path.push("index.html");
                        if let Ok(file) = File::open(&path) {
                            let filename = path.file_name().as_ref().unwrap().to_str().unwrap();
                            let ext = Path::new(filename).extension().unwrap().to_str().unwrap();
                            res.file(file, ContentType::from(ext));
                        } else {
                            res.json("no such file");
                        }
                    } else {
                        let filename = path.file_name().as_ref().unwrap().to_str().unwrap();
                        let ext = Path::new(filename).extension().unwrap().to_str().unwrap();
                        res.file(file, ContentType::from(ext));
                    }
                } else {
                    res.json("no such file");
                }
            });
    }

}

impl HttpHander for HttpServer {
    fn post<T>(&mut self, url: &str, controller: T)
    where
        T: Fn(HttpRequest, HttpResponse) + Sync + Send + 'static,
    {
        self.controller
            .as_mut()
            .unwrap()
            .add_handler("POST", url, Box::new(controller));
    }
    fn get<T>(&mut self, url: &str, controller: T)
    where
        T: Fn(HttpRequest, HttpResponse) + Sync + Send + 'static,
    {
        self.controller
            .as_mut()
            .unwrap()
            .add_handler("GET", url, Box::new(controller));
    }
    fn router(&mut self, url: &str, route: route::Route) {
        let self_controller = self.controller.as_mut().unwrap();
        let controller = route.get_controller();
        let mut handlers: std::collections::HashMap<String, std::collections::HashMap<String, Box<dyn Fn(HttpRequest, HttpResponse) + Send + Sync>>> = controller.take();
        let methods = handlers.keys().map(|key| key.to_string()).collect::<Vec<String>>();
        for method in methods{
            let mut handers_inner = handlers.remove(method.as_str()).unwrap();
            let url_inner = handers_inner.keys().map(|key| key.to_string()).collect::<Vec<String>>();
            for handler_inner in url_inner {
                let handler: Box<dyn Fn(HttpRequest, HttpResponse) + Send + Sync> = handers_inner.remove(handler_inner.as_str()).unwrap();
                self_controller.add_handler(method.as_str(), format!("{}{}",url,handler_inner).as_str(), handler);
            }
        }
    }
}
pub fn handle_stream(stream: TcpStream, controller: Arc<Controller>) {
    let rc = Rc::new(RefCell::new(stream));
    let request = match HttpRequest::build(rc.clone()) {
        Ok(req) => req,
        Err(e) => {
            println!("Error: {}", e);
            let response = HttpResponse::init(rc.clone(), "HTTP/1.1");
            controller.handle_not_found(response);
            return;
        }
    };
    let response = HttpResponse::init(rc.clone(), request.version());
    controller.handle_request(request, response);
}
