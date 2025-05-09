use std::error::Error;

use http::{Request, Response};
use http_server_async_czh::CzhServer;

async fn hello(req: Request<String>) -> Response<String> {
    Response::new("body".to_string())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let server = CzhServer::builder()
        .post("/hello", async |req| Response::new("body".to_string()))
        .post("/a", hello)
        .build()
        .start()
        .await;
    Ok(())
}
