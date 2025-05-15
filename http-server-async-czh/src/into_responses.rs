use http::Response;

use crate::body_type::ResponseBody;

pub trait IntoResponse {
    fn into_response(self) -> Response<ResponseBody>;
}

impl <T:AsRef<str>> IntoResponse for T {
    fn into_response(self) -> Response<ResponseBody> {
        ResponseBody::Text(self.as_ref().as_bytes().to_vec()).into()
    }
}
impl IntoResponse for ResponseBody {
    fn into_response(self) -> Response<ResponseBody> {
        self.into()
    }
}
