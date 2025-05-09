use std::collections::HashMap;

use actor::{ProcessActor, ResponseHandle, Routes};
use http::{Method, Request, Response};

pub mod actor;

pub mod encoder;

pub struct CzhServer {
    process_actor: ProcessActor,
}

impl CzhServer {
    pub fn builder() -> CzhServerBuilder {
        CzhServerBuilder {
            routes: HashMap::new(),
        }
    }

    pub async fn start(self) {
        // start process_actor
        tokio::spawn(async move { self.process_actor.run().await });
        // start response_actor
    }
}

pub struct CzhServerBuilder {
    routes: Routes,
}
impl CzhServerBuilder {
    pub fn post<T, F>(mut self, path: &'static str, f: T) -> Self
    where
        T: 'static + Fn(Request<String>) -> F + Send,
        F: Future<Output = Response<String>> + 'static + Send,
    {
        if let Some(e) = self.routes.get_mut(&Method::POST) {
            e.insert(path, Box::new(move |req| Box::pin(f(req))));
        } else {
        }
        self
    }
    pub fn build(mut self) -> CzhServer {
        let (s, r) = tokio::sync::mpsc::channel(10);
        CzhServer {
            process_actor: ProcessActor::new(self.routes, r, ResponseHandle::new()),
        }
    }
}
