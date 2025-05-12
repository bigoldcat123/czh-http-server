use std::path::PathBuf;


pub enum ResponseBody {
    File(PathBuf),
    Text(Vec<u8>),
    Json(Vec<u8>),
}