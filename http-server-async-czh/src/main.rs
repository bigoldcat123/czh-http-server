use std::error::Error;

use http::{Request, Response};
use http_server_async_czh::CzhServer;

async fn hello(req: Request<String>) -> Response<String> {
    Response::new("body".to_string())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let _ = CzhServer::builder()
        .post("/hello", async |_| Response::new("body".to_string()))
        .post("/a", hello)
        .posts(vec![
            (
                "e",
                Box::new(|req: Request<String>| {
                    Box::pin(async move { Response::new(String::new()) })
                }),
            ),
            (
                "e",
                Box::new(|req: Request<String>| {
                    Box::pin(async move { Response::new(String::new()) })
                }),
            ),
        ])
        .build()
        .start()
        .await;
    Ok(())
}
