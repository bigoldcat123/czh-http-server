use std::{collections::HashMap, error::Error};

use actor::{ProcessActor, ProcessHandle, ResponseActor, ResponseHandle, RouteHandler, Routes};
use decoder::RequestDecoder;
use futures::StreamExt;
use http::{Method, Request, Response};
use tokio::net::TcpListener;
use tokio_util::codec::{FramedRead, LinesCodec};

pub mod actor;

pub mod decoder;
pub mod encoder;

pub struct CzhServer {
    process_actor: ProcessActor,
    process_handle: ProcessHandle,
}

impl CzhServer {
    pub fn builder() -> CzhServerBuilder {
        CzhServerBuilder {
            routes: HashMap::new(),
        }
    }

    pub async fn start(self) -> Result<(), Box<dyn Error>> {
        // start process_actor
        tokio::spawn(async move { self.process_actor.run().await });
        // start response_actor

        //

        let server = TcpListener::bind("localhost:7788").await?;
        while let Ok((mut client, _)) = server.accept().await {
            let mut process_handle = self.process_handle.clone();
            let (stream, sink) = client.into_split();


            let Response_actor = ResponseActor::new(sink);

            tokio::spawn(async move {
                let mut stream = FramedRead::new(stream, RequestDecoder::new());
                while let Some(Ok(next)) = stream.next().await {
                    let _ = process_handle.send((next, ResponseHandle {})).await;
                }
            });
        }
        Ok(())
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
    pub fn posts(self, vk: Vec<(&'static str, RouteHandler)>) -> Self {
        let mut other = self;
        for (v, k) in vk {
            if let Some(e) = other.routes.get_mut(&Method::POST) {
                e.insert(v, Box::new(move |req| k(req)));
            }
            other = other;
        }
        other
    }
    pub fn build(self) -> CzhServer {
        let (process_sender, process_reciver) = tokio::sync::mpsc::channel(10);
        CzhServer {
            process_actor: ProcessActor::new(self.routes, process_reciver),
            process_handle: ProcessHandle::new(process_sender),
        }
    }
}
