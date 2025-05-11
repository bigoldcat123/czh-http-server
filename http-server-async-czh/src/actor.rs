use std::{collections::HashMap, pin::Pin, sync::Arc};

use futures::SinkExt;
use http::{Method, Request, Response};
use tokio::{
    net::tcp::OwnedWriteHalf,
    sync::mpsc::{Receiver, Sender, error::SendError},
};
use tokio_util::codec::FramedWrite;

use crate::encoder::ResponseEncoder;

pub type RouteHandler = Box<
    dyn Fn(
            Request<String>,
        ) -> Pin<Box<dyn Future<Output = Response<String>> + Send + 'static + Sync>>
        + 'static
        + Send
        + Sync,
>;
pub type Routes = HashMap<Method, HashMap<&'static str, RouteHandler>>;
pub type SharedRoutes = HashMap<Method, Arc<HashMap<&'static str, Arc<RouteHandler>>>>;

pub struct ProcessActor {
    routes: SharedRoutes,
    receiver: Receiver<(Request<String>, ResponseHandle)>,
}
impl ProcessActor {
    pub fn new(
        routes: SharedRoutes,
        receiver: Receiver<(Request<String>, ResponseHandle)>,
    ) -> Self {
        Self { receiver, routes }
    }
    pub async fn run(mut self) {
        while let Some(r) = self.receiver.recv().await {
            if let Some(e) = self.routes.get(r.0.method()) {
                if let Some(m) = Arc::clone(e).get("k") {
                    let m = Arc::clone(m);
                    tokio::spawn(async move {
                        m(r.0).await;
                    });
                }
            }
        }
    }
}
#[derive(Clone)]
pub struct ProcessHandle {
    sender: Sender<(Request<String>, ResponseHandle)>,
}
impl ProcessHandle {
    pub fn new(sender: Sender<(Request<String>, ResponseHandle)>) -> Self {
        Self { sender }
    }
    pub async fn send(
        &mut self,
        req: (Request<String>, ResponseHandle),
    ) -> Result<(), SendError<(Request<String>, ResponseHandle)>> {
        self.sender.send(req).await?;
        Ok(())
    }
}
pub struct ResponseActor {
    sink: FramedWrite<OwnedWriteHalf, ResponseEncoder>,
    receiver: Receiver<Response<String>>,
}
impl ResponseActor {
    pub fn new(sink: OwnedWriteHalf, receiver: Receiver<Response<String>>) -> Self {
        let sink = FramedWrite::new(sink, ResponseEncoder::new());
        Self { sink, receiver }
    }

    pub async fn run(mut self) {
        while let Some(n) = self.receiver.recv().await {
            let _ = self.sink.send(n).await;
        }
    }
}
#[derive(Clone)]
pub struct ResponseHandle {
    sender: Sender<Response<String>>,
}

impl ResponseHandle {
    pub fn new(sender: Sender<Response<String>>) -> Self {
        Self { sender }
    }
    pub async fn send(&mut self, res: Response<String>) -> Result<(), SendError<Response<String>>> {
        self.sender.send(res).await
    }
}
