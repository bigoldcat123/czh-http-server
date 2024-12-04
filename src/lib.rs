//!
//! # czh_http_server
//!
//! czh_http_server is a simple http server
//! # Example
//! ```
//! let mut server  = HttpServer::create_server("localhost", 3000);
//!    // server.listen();
//!    server.filter("/home",|req,res| {
//!        println!("{:#?}","hello i am filterb");
//!        if req.url() == "/home/abc" {
//!            res.json("GLALALALALALA");
//!            return None
//!        }
//!        Some((req,res))
//!    });
//!    server.map("/file","/Users/dadigua/Desktop/lifetime/app/nextjs-static/dist");
//!
//!    server.get("/home",|req,mut res| {
//!        println!("{:#?}",req.url());
//!        // println!("{:#?}",req.headers());
//!        println!("{:#?}",req.cookies());
//!        res.set_cookie("cooooooo", "this is cookie setted by server");
//!        res.json("hello fetch");
//!    });
//!    server.get("/home/abc",|req,res| {
//!        println!("{:#?}",req.url());
//!        res.json("hello fetch/ home/abc");
//!    });
//!    
//!    server.post("/post",|mut req,res| {
//!        match req.json::<Student>() {
//!            Ok(stu) => {
//!                println!("{:#?}",stu);
//!            },
//!            Err(_) => {
//!                res.json("error parse json");
//!                return;
//!            },
//!        }
//!        println!("{:#?}",req.url());
//!        res.json("hello post");
//!    });
//!
//!    let mut route = Route::new();
//!
//!    route.get("/sayhello", |req, res| {
//!        // req.url()
//!        println!("{:#?}",req.url());
//!        res.json(Student{
//!            name:"dadigua".to_string(),
//!            age:18
//!        });
//!    });
//!
//!    server.router("/route",route);
//!    server.listen();
//! ```
//!
use std::{
    cell::RefCell, fs::File, mem, net::{TcpListener, TcpStream}, path::{Path, PathBuf}, rc::Rc, sync::Arc
};

use controller::Controller;
use filter::{Filter, FilterChain};
use request::HttpRequest;
use response::{ContentType, HttpResponse};
mod controller;
pub mod request;
pub mod response;
pub mod route;
pub mod filter;
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


///
/// # HttpServer
/// used to create a HTTP_SERVER
/// 
/// # Example
/// ```
/// let mut server  = HttpServer::create_server("localhost", 3000);
/// server.get("/hello",|req,mut res| {
///     println!("{:#?}","hello i am filterb");
///     res.json("hello fetch");    
/// })
/// server.listen();
/// ```
pub struct HttpServer {
    listener: TcpListener,
    // port: u16,
    controller: Option<Controller>,
    filter_chain:FilterChain
}
impl HttpServer {
    pub fn create_server(host: &str, port: u16) -> Self {
        let listener = TcpListener::bind(format!("{}:{}", host, port)).unwrap();
        HttpServer {
            listener,
            controller: Some(Controller::new()),
            filter_chain: FilterChain::new()
        }
    }
    /// start listen request
    /// 
    /// # Example
    /// ```
    ///  let mut server  = HttpServer::create_server("localhost", 3000);
    ///  server.listen();
    /// ```
    pub fn listen(mut self) {
        let controller = Arc::new(self.controller.take().unwrap());
        let filter  = Arc::new(self.filter_chain);
        let pool = czhmt::ThreadPool::new(4);
        for client in self.listener.incoming() {
            print!("hello");
            match client {
                Ok(stream) => {
                    let controller = controller.clone();
                    let filter = filter.clone();
                    pool.exec(|| {
                        handle_stream(stream, controller,filter);
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
    
    pub fn filter(&mut self, url: &str, filter: impl Fn(HttpRequest,HttpResponse) -> Option<(HttpRequest,HttpResponse)> + Send + Sync + 'static) {
        self.filter_chain.add_filter(Filter::new(filter),url);
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
pub fn handle_stream(stream: TcpStream, controller: Arc<Controller>,filter:Arc<FilterChain>) {
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
    if let Some((request, response)) = filter.exec(request, response)  {
        controller.handle_request(request, response);
    }else {

    }

    
}
