use std::error::Error;
use std::path::PathBuf;

use http::{Method, Request};
use http_server_async_czh::CzhServer;
use http_server_async_czh::body_type::ResponseBody::{self, File, Json};
use log::info;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Student {
    name: String,
    age: u16,
}

async fn hello(_: Request<String>) -> ResponseBody {
    File(PathBuf::from(
        "/Users/dadigua/Desktop/czh-http-server/http-server-async-czh/src/main.rs",
    ))
}
async fn guard_hello1(req: Request<String>) -> (Request<String>, Option<String>) {
    info!("i am a guard!2222222");
    (req, None)
}
async fn guard_hello(req: Request<String>) -> (Request<String>, Option<String>) {
    info!("i am a guard!");
    // (
    //     req,
    //     Some(
    //         Json(
    //             serde_json::to_vec(&Student {
    //                 name: String::from("you are not allowed!"),
    //                 age: 12,
    //             })
    //             .unwrap(),
    //         )
    //         .into(),
    //     ),
    // )
    (req, None)
}
async fn hello2(_: Request<String>) -> ResponseBody {
    let s = Student {
        name: String::from("hello"),
        age: 18,
    };
    let s = serde_json::to_vec(&s).unwrap();
    Json(s)
}
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let _ = CzhServer::builder()
        .get("/", hello)
        .guard_at(Method::GET, "/", guard_hello1)
        .guard_at(Method::GET, "/", guard_hello)
        .get("/stu", hello2)
        .build()
        .start()
        .await;
    Ok(())
}
