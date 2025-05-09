use std::{collections::HashMap, pin::Pin};

use http::{Method, Request, Response};
use tokio::{io::AsyncWrite, sync::mpsc::Receiver};
use tokio_util::codec::FramedWrite;

use crate::encoder::ResponseEncoder;

pub type Routes = HashMap<
    Method,
    HashMap<
        &'static str,
        Box<
            dyn Fn(
                Request<String>,
            ) -> Pin<Box<dyn Future<Output = Response<String>> + Send + 'static>>,
        >,
    >,
>;

pub struct ProcessActor {
    routes: Routes,
    receiver: Receiver<Request<String>>,
}
impl ProcessActor {
    pub async fn run(mut self) {
        while let Some(r) = self.receiver.recv().await {
            if let Some(e) = self.routes.get(r.method()) {
                if let Some(m) = e.get("k") {
                    let _ = m(r).await;
                }
            }
        }
    }
}
struct ProcessHandle {}
pub struct ResponseActor<T>
where
    T: AsyncWrite,
{
    routes: HashMap<Method, HashMap<String, FramedWrite<T, ResponseEncoder>>>,
    receiver: Receiver<(Method, String, Response<String>)>,
}

impl<T: AsyncWrite + Unpin + Send + 'static> ResponseActor<T> {
    pub fn new() -> Self {
        let (sender, receiver) = tokio::sync::mpsc::channel(10);
        let (sender2, mut receiver2) = tokio::sync::mpsc::channel::<T>(10);
        tokio::spawn(async move {
            let e = receiver2.recv().await.unwrap();
            let e = FramedWrite::new(e, ResponseEncoder::new());
            let mut ee = Self::new();
            ee.routes
                .get_mut(&Method::POST)
                .unwrap()
                .insert("k".to_string(), e);
        });
        Self {
            routes: HashMap::new(),
            receiver: receiver,
        }
    }
    pub async fn run(mut self) {
        while let Some((m, p, r)) = self.receiver.recv().await {
            if let Some(rts) = self.routes.get_mut(&m) {
                if let Some(sink) = rts.get_mut(&p) {
                    r.body().bytes().into_iter();
                }
            }
        }
    }
}
struct ResponseHandle {}
