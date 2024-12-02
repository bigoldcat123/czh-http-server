//!
//! # czh_http_server
//!
//! czh_http_server is a simple http server
//!
use std::{
    cell::RefCell,
    fs::File,
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};

use controller::Controller;
use request::HttpRequest;
use response::{ContentType, HttpResponse};
mod controller;
pub mod request;
pub mod response;
type ControllerHandler = Box<dyn Fn(HttpRequest, HttpResponse) + Sync + Send + 'static>;

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

    pub fn get<T>(&mut self, url: &str, controller: T)
    where
        T: Fn(HttpRequest, HttpResponse) + Sync + Send + 'static,
    {
        self.controller
            .as_mut()
            .unwrap()
            .add_handler("GET", url, controller);
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
                    // if file.metadata().unwrap().is_dir(){
                    //     res.json("404");
                    //     return;
                    // }else {
                    //     let filename = path.file_name().as_ref().unwrap().to_str().unwrap();
                    //     let ext = Path::new(filename).extension().unwrap().to_str().unwrap();
                    //     res.file(file,ContentType::from(ext));
                    // }
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
    
    pub fn post<T>(&mut self, url: &str, controller: T)
    where
        T: Fn(HttpRequest, HttpResponse) + Sync + Send + 'static,
    {
        self.controller
            .as_mut()
            .unwrap()
            .add_handler("POST", url, controller);
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
