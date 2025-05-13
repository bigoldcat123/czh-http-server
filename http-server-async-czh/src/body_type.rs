use std::path::PathBuf;

use http::Response;

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
