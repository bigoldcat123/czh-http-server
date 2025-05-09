use std::{collections::HashMap, io::Write};

use actor::Routes;
use http::{Method, Request, Response};

pub mod actor;

pub mod encoder;

pub struct CzhServer {}

impl CzhServer {
    pub fn builder() -> CzhServerBuilder {
        CzhServerBuilder {
            routes: HashMap::new(),
        }
    }
}

pub struct CzhServerBuilder {
    routes: Routes,
}
impl CzhServerBuilder {
    pub async fn post<T, F>(mut self, path: &'static str, f: T) -> Self
    where
        T: 'static + Fn(Request<String>) -> F,
        F: Future<Output = Response<String>> + 'static + Send,
    {
        if let Some(e) = self.routes.get_mut(&Method::POST) {
            e.insert(path, Box::new(move |req| Box::pin(f(req))));
        } else {
        }
        self
    }
}
