use std::{collections::HashMap, pin::Pin, sync::Arc};

use futures::SinkExt;
use http::{Method, Request, Response, StatusCode};
use log::{error, info};
use tokio::{
    net::tcp::OwnedWriteHalf,
    sync::mpsc::{Receiver, Sender, error::SendError},
};
use tokio_util::codec::FramedWrite;

use crate::{body_type::{RequestBody, ResponseBody}, encoder::ResponseEncoder, into_responses::IntoResponse};

pub type RouteHandler = Box<
    dyn Fn(
            Request<RequestBody>,
        ) -> Pin<Box<dyn Future<Output = Response<ResponseBody>> + Send + 'static + Sync>>
        + 'static
        + Send
        + Sync,
>;
pub type RouteGuard = Box<
    dyn Fn(
            Request<RequestBody>,
        ) -> Pin<
            Box<
                dyn Future<Output = (Request<RequestBody>, Option<Response<ResponseBody>>)>
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

/// ## response to process request
/// receive request from handle, and execute it
pub struct ProcessActor {
    routes: SharedRoutes,
    routes_guards: SharedGuards,
    receiver: Receiver<(Request<RequestBody>, ResponseHandle)>,
}
impl ProcessActor {
    pub fn new(
        routes: SharedRoutes,
        guards: SharedGuards,
        receiver: Receiver<(Request<RequestBody>, ResponseHandle)>,
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
        req: Request<RequestBody>,
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

                        if res.1.is_none() {
                            // Self::handle_req(res.0, response_handle, routes);
                            req = Some(res.0);
                        } else {
                            info!("6. send response to response actor guard worked!");
                            let _ = response_handle.send(res.1.unwrap()).await;
                            break;
                        }
                    }
                    if req.is_some() {
                        info!("6. send response to response actor");
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

    fn handle_req(
        req: Request<RequestBody>,
        mut response_handle: ResponseHandle,
        routes: &SharedRoutes,
    ) {
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
                error!("no such path {}", req.uri().path());
                tokio::spawn(async move {
                    response_handle
                        .send({
                            let mut res = "not found!".into_response();
                            *res.status_mut() = StatusCode::NOT_FOUND;
                            res
                        })
                        .await
                });
            }
        } else {
            error!("no such method {}", req.method());
            tokio::spawn(async move {
                response_handle
                    .send({
                        let mut res = "method not supported !!".into_response();
                        *res.status_mut() = StatusCode::NOT_FOUND;
                        res
                    })
                    .await
            });
        }
    }
}
#[derive(Clone)]
pub struct ProcessHandle {
    sender: Sender<(Request<RequestBody>, ResponseHandle)>,
}
impl ProcessHandle {
    pub fn new(sender: Sender<(Request<RequestBody>, ResponseHandle)>) -> Self {
        Self { sender }
    }
    pub async fn send(
        &mut self,
        req: (Request<RequestBody>, ResponseHandle),
    ) -> Result<(), SendError<(Request<RequestBody>, ResponseHandle)>> {
        self.sender.send(req).await?;
        Ok(())
    }
}

/// ## receive response and send it to sink
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

/// used to send response to **ResponseActor**
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
