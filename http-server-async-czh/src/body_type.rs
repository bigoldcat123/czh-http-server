use std::path::PathBuf;

use http::Response;
use serde::{Deserialize, Serialize};
use tokio::fs::File;

pub enum ResponseBody {
    File(PathBuf),
    Text(Vec<u8>),
    Json(Vec<u8>),
}

impl Into<Response<ResponseBody>> for ResponseBody {
    fn into(self) -> Response<ResponseBody> {
        let x = Response::builder().body(self).expect("into error");
        x
    }
}
pub struct JsonBody {
    inner: String,
}
impl JsonBody {
    pub fn new(inner: String) -> Self {
        Self { inner }
    }
    pub fn data<'a, T: Deserialize<'a>>(&'a self) -> T {
        #[derive(Serialize, Deserialize)]
        struct A {
            name: String,
        }
        serde_json::from_str(self.inner.as_str()).unwrap()
    }
}

pub enum RequestBody {
    File(File),
    Text(String),
    Json(JsonBody),
}
