use std::{collections::HashMap, pin::Pin, sync::Arc};

use futures::SinkExt;
use http::{Method, Request, Response};
use log::{error, info};
use tokio::{
    net::tcp::OwnedWriteHalf,
    sync::mpsc::{Receiver, Sender, error::SendError},
};
use tokio_util::codec::FramedWrite;

use crate::{body_type::ResponseBody, encoder::ResponseEncoder};

pub type RouteHandler = Box<
    dyn Fn(
            Request<String>,
        ) -> Pin<Box<dyn Future<Output = Response<ResponseBody>> + Send + 'static + Sync>>
        + 'static
        + Send
        + Sync,
>;
pub type RouteGuard = Box<
    dyn Fn(
            Request<String>,
        ) -> Pin<
            Box<
                dyn Future<Output = (Request<String>, Option<Response<ResponseBody>>)>
                    + Send
                    + 'static
                    + Sync,
            >,
        >
        + 'static
        + Send
        + Sync,
>;
pub type Routes = HashMap<Method, HashMap<&'static str, RouteHandler>>;
pub type Guards = HashMap<Method, HashMap<&'static str, Vec<RouteGuard>>>;

pub type SharedRoutes = HashMap<Method, Arc<HashMap<&'static str, Arc<RouteHandler>>>>;
pub type SharedGuards = HashMap<Method, Arc<HashMap<&'static str, Arc<Vec<RouteGuard>>>>>;

pub struct ProcessActor {
    routes: SharedRoutes,
    routes_guards: SharedGuards,
    receiver: Receiver<(Request<String>, ResponseHandle)>,
}
impl ProcessActor {
    pub fn new(
        routes: SharedRoutes,
        guards: SharedGuards,
        receiver: Receiver<(Request<String>, ResponseHandle)>,
    ) -> Self {
        Self {
            receiver,
            routes,
            routes_guards: guards,
        }
    }
    pub async fn run(mut self) {
        while let Some((req, response_handle)) = self.receiver.recv().await {
            info!("4. receive parsed req");
            Self::guard(req, response_handle, &self.routes, &self.routes_guards);
        }
    }

    fn guard(
        req: Request<String>,
        response_handle: ResponseHandle,
        routes: &SharedRoutes,
        guards: &SharedGuards,
    ) {
        if let Some(e) = guards.get(req.method()) {
            if let Some(m) = Arc::clone(e).get(req.uri().path()) {
                let m = Arc::clone(m);
                let mut response_handle = response_handle.clone();

                let e = Arc::clone(
                    routes
                        .get(req.method())
                        .unwrap()
                        .get(req.uri().path())
                        .unwrap(),
                );

                tokio::spawn(async move {
                    info!("5. exec routes handler");
                    let mut req = Some(req);
                    for m in m.iter() {
                        let res = m(req.take().unwrap()).await;
                        info!("6. send response to response actor");
                        if res.1.is_none() {
                            // Self::handle_req(res.0, response_handle, routes);
                            req = Some(res.0);
                        } else {
                            let _ = response_handle.send(res.1.unwrap()).await;
                        }
                    }
                    if req.is_some() {
                        let res = e(req.take().unwrap()).await;
                        let _ = response_handle.send(res).await;
                    }
                });
            } else {
                Self::handle_req(req, response_handle, routes);
            }
        } else {
            Self::handle_req(req, response_handle, routes);
        }
    }

    fn handle_req(req: Request<String>, response_handle: ResponseHandle, routes: &SharedRoutes) {
        if let Some(e) = routes.get(req.method()) {
            if let Some(m) = Arc::clone(e).get(req.uri().path()) {
                let m = Arc::clone(m);
                let mut response_handle = response_handle.clone();
                tokio::spawn(async move {
                    info!("5. exec routes handler");
                    let res = m(req).await;
                    info!("6. send response to response actor");
                    let _ = response_handle.send(res).await;
                });
            } else {
                error!("no such puth {}", req.uri().path());
            }
        } else {
            error!("no such method {}", req.method());
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
    receiver: Receiver<Response<ResponseBody>>,
}
impl ResponseActor {
    pub fn new(sink: OwnedWriteHalf, receiver: Receiver<Response<ResponseBody>>) -> Self {
        let sink = FramedWrite::new(sink, ResponseEncoder::new());
        Self { sink, receiver }
    }

    pub async fn run(mut self) {
        while let Some(n) = self.receiver.recv().await {
            info!("7. serialize response to sink");
            let _ = self.sink.send(n).await;
        }
        info!("response actor is out~");
    }
}
#[derive(Clone)]
pub struct ResponseHandle {
    sender: Sender<Response<ResponseBody>>,
}

impl ResponseHandle {
    pub fn new(sender: Sender<Response<ResponseBody>>) -> Self {
        Self { sender }
    }
    pub async fn send(
        &mut self,
        res: Response<ResponseBody>,
    ) -> Result<(), SendError<Response<ResponseBody>>> {
        self.sender.send(res).await
    }
}
