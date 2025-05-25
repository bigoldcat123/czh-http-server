use std::{collections::HashMap, error::Error, process, sync::Arc};

use actor::{
    Guards, ProcessActor, ProcessHandle, ResponseActor, ResponseHandle, Routes, SharedGuards,
    SharedRoutes,
};
use decoder::RequestDecoder;
use futures::StreamExt;
use http::{Method, Request};
use into_responses::IntoResponse;
use log::{error, info};
use tokio::net::TcpListener;
use tokio_util::codec::FramedRead;

pub mod actor;

pub mod body_type;
pub mod decoder;
pub mod encoder;
pub mod into_responses;

/// ## run *ProcessActor* and send req to it by *ProcessHandle*
pub struct CzhServer {
    process_actor: ProcessActor,
    process_handle: ProcessHandle,
}

impl CzhServer {
    pub fn builder() -> CzhServerBuilder {
        CzhServerBuilder {
            routes: HashMap::new(),
            guards: HashMap::new(),
        }
    }

    pub async fn start(self) -> Result<(), Box<dyn Error>> {
        // start process_actor
        info!("starting <ProcessActor>");
        tokio::spawn(async move { self.process_actor.run().await });
        // start response_actor

        //
        info!("starting <TCP Server>");
        Self::start_server(self.process_handle).await?;

        info!("Sever is started!");
        Ok(())
    }
    async fn start_server(process_handle: ProcessHandle) -> Result<(), Box<dyn Error>> {
        let server = TcpListener::bind("localhost:7788").await?;
        info!("start at localhost:7788");
        while let Ok((client, _)) = server.accept().await {
            info!("1. received req");
            let mut process_handle = process_handle.clone();
            let (stream, sink) = client.into_split();

            let (sender, receiver) = tokio::sync::mpsc::channel(10);
            let response_actor = ResponseActor::new(sink, receiver);

            info!("2. start response_actor");
            tokio::spawn(response_actor.run());

            tokio::spawn(async move {
                let mut stream = FramedRead::new(stream, RequestDecoder::new());
                let response_handle = ResponseHandle::new(sender);
                while let Some(Ok(next)) = stream.next().await {
                    info!("3. parse stream to req instanse");
                    let _ = process_handle.send((next, response_handle.clone())).await;
                }
                info!("connection over~!!!!!");
            });
        }
        Ok(())
    }
}

pub struct CzhServerBuilder {
    routes: Routes,
    guards: Guards,
}
impl CzhServerBuilder {
    fn insert_route_hadnler<T, F, O>(&mut self, method: Method, path: &'static str, f: T)
    where
        T: 'static + Copy + Fn(Request<String>) -> F + Send + Sync,
        F: Future<Output = O> + 'static + Send + Sync,
        O: IntoResponse,
    {
        if let Some(e) = self.routes.get_mut(&method) {
            e.insert(
                path,
                Box::new(move |req| Box::pin(async move { f(req).await.into_response() })),
            );
        } else {
            let new_map = HashMap::new();
            self.routes.insert(method.clone(), new_map);
            self.insert_route_hadnler(method, path, f);
        }
    }

    fn routes_exists(&self, method: &Method, path: &'static str) -> bool {
        if let Some(e) = self.routes.get(method) {
            if let Some(_) = e.get(path) {
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn guard_at<T, F, O>(mut self, method: Method, path: &'static str, f: T) -> Self
    where
        T: 'static + Copy + Fn(Request<String>) -> F + Send + Sync,
        F: Future<Output = (Request<String>, Option<O>)> + 'static + Send + Sync,
        O: IntoResponse,
    {
        if !self.routes_exists(&method, path) {
            error!("guard must be added on an existed route!");
            process::exit(1);
        }
        if let Some(e) = self.guards.get_mut(&method) {
            if let Some(v) = e.get_mut(path) {
                v.push(Box::new(move |req| {
                    Box::pin(async move {
                        let x = f(req).await;
                        if x.1.is_none() {
                            (x.0, None)
                        } else {
                            (x.0, Some(x.1.unwrap().into_response()))
                        }
                    })
                }));
            } else {
                e.insert(path, vec![]);
                return self.guard_at(method, path, f);
            }
            self
        } else {
            let new_map = HashMap::new();
            self.guards.insert(method.clone(), new_map);
            self.guard_at(method, path, f)
        }
    }

    pub fn post<T, F>(mut self, path: &'static str, f: T) -> Self
    where
        T: 'static + Fn(Request<String>) -> F + Send + Sync + Copy,
        F: Future + 'static + Send + Sync,
        F::Output: IntoResponse,
    {
        self.insert_route_hadnler(Method::POST, path, f);
        self
    }
    pub fn get<T, F>(mut self, path: &'static str, f: T) -> Self
    where
        T: 'static + Fn(Request<String>) -> F + Send + Sync + Copy,
        F: Future + 'static + Send + Sync,
        F::Output: IntoResponse,
    {
        self.insert_route_hadnler(Method::GET, path, f);
        self
    }
    // pub fn posts(self, vk: Vec<(&'static str, RouteHandler)>) -> Self {
    //     let mut other = self;
    //     for (v, k) in vk {
    //         if let Some(e) = other.routes.get_mut(&Method::POST) {
    //             e.insert(v, Box::new(move |req| k(req)));
    //         }
    //         other = other;
    //     }
    //     other
    // }
    pub fn build(self) -> CzhServer {
        let (process_sender, process_reciver) = tokio::sync::mpsc::channel(10);
        CzhServer {
            process_actor: ProcessActor::new(
                convert2shared(self.routes),
                convert2shared_guard(self.guards),
                process_reciver,
            ),
            process_handle: ProcessHandle::new(process_sender),
        }
    }
}

fn convert2shared(routes: Routes) -> SharedRoutes {
    let mut res = HashMap::new();

    routes.into_iter().for_each(|(k, v)| {
        let mut new_map = HashMap::new();
        v.into_iter().for_each(|(innder_k, inner_v)| {
            new_map.insert(innder_k, Arc::new(inner_v));
        });
        res.insert(k, Arc::new(new_map));
    });
    res
}

fn convert2shared_guard(routes: Guards) -> SharedGuards {
    let mut res = HashMap::new();

    routes.into_iter().for_each(|(k, v)| {
        let mut new_map = HashMap::new();
        v.into_iter().for_each(|(innder_k, inner_v)| {
            new_map.insert(innder_k, Arc::new(inner_v));
        });
        res.insert(k, Arc::new(new_map));
    });
    res
}
