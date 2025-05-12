use std::{error::Error, time::Duration};

use http::{HeaderValue, Request, Response};
use http_server_async_czh::CzhServer;
use http_server_async_czh::body_type::ResponseBody;
use log::info;
use tokio::time;

async fn hello(_: Request<String>) -> Response<ResponseBody> {
    Response::new(ResponseBody::Text("body".as_bytes().to_vec()))
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let _ = CzhServer::builder()
        .post("/", async |_| {
            info!("i am executed");
            time::sleep(Duration::from_secs(3)).await;
            let res = "hello".to_string();
            Response::builder()
                // .header(
                //     "content-length",
                //     HeaderValue::from_str(&res.as_bytes().len().to_string()).unwrap(),
                // )
                .body(ResponseBody::Text(res.as_bytes().to_vec()))
                .unwrap()
        })
        .post("/a", hello)
        .build()
        .start()
        .await;
    Ok(())
}
