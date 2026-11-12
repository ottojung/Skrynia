use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Debug)]
pub struct Fail(pub StatusCode, pub &'static str);
impl IntoResponse for Fail {
    fn into_response(self) -> Response {
        (self.0, axum::Json(serde_json::json!({"error": self.1}))).into_response()
    }
}
pub type Result<T> = std::result::Result<T, Fail>;
pub fn bad(code: &'static str) -> Fail {
    Fail(StatusCode::BAD_REQUEST, code)
}
pub fn forbidden(code: &'static str) -> Fail {
    Fail(StatusCode::FORBIDDEN, code)
}
pub fn missing(code: &'static str) -> Fail {
    Fail(StatusCode::NOT_FOUND, code)
}
pub fn internal() -> Fail {
    Fail(StatusCode::INTERNAL_SERVER_ERROR, "internal_error")
}
