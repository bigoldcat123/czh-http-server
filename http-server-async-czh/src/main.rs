use std::error::Error;
use std::path::PathBuf;

use http::{Method, Request, Response};
use http_server_async_czh::CzhServer;
use http_server_async_czh::body_type::ResponseBody::{self, File, Json};
use log::info;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Student {
    name: String,
    age: u16,
}

async fn hello(_: Request<String>) -> Response<ResponseBody> {
    File(PathBuf::from(
        "/Users/dadigua/Desktop/czh-http-server/http-server-async-czh/src/main.rs",
    ))
    .into()
}

async fn guard_hello(req: Request<String>) -> (Request<String>, Option<Response<ResponseBody>>) {
    info!("i am a guard!");
    (
        req,
        Some(
            Json(
                serde_json::to_vec(&Student {
                    name: String::from("you are not allowed!"),
                    age: 12,
                })
                .unwrap(),
            )
            .into(),
        ),
    )
}
async fn hello2(_: Request<String>) -> Response<ResponseBody> {
    let s = Student {
        name: String::from("hello"),
        age: 18,
    };
    let s = serde_json::to_vec(&s).unwrap();
    Json(s).into()
}
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let _ = CzhServer::builder()
        .get("/", hello)
        .guard_at(Method::GET, "/", guard_hello)
        .get("/stu", hello2)
        .build()
        .start()
        .await;
    Ok(())
}
