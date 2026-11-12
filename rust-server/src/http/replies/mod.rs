mod token;
use axum::body::Body;
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
pub use token::token;

pub fn json(status: StatusCode, value: serde_json::Value) -> Response {
    (status, axum::Json(value)).into_response()
}
pub fn ok(value: serde_json::Value) -> Response {
    json(StatusCode::OK, value)
}
pub fn cors(reply: &mut Response) {
    let headers = reply.headers_mut();
    headers.insert("access-control-allow-origin", "*".parse().unwrap());
    headers.insert("access-control-expose-headers", "ETag".parse().unwrap());
    if !headers.contains_key("cache-control")
        && headers
            .get("content-type")
            .is_some_and(|v| v.to_str().is_ok_and(|s| s.starts_with("application/json")))
    {
        headers.insert("cache-control", HeaderValue::from_static("no-store"));
    }
}
pub fn preflight() -> Response {
    let mut response = Response::new(Body::empty());
    *response.status_mut() = StatusCode::NO_CONTENT;
    let headers = response.headers_mut();
    headers.insert(
        "access-control-allow-methods",
        "GET, POST, PUT, DELETE, OPTIONS".parse().unwrap(),
    );
    headers.insert("access-control-allow-headers", "Authorization, Content-Type, X-Skrynia-Capability, X-Skrynia-If-SHA256, If-None-Match, If-Range, Range".parse().unwrap());
    headers.insert("access-control-max-age", "3600".parse().unwrap());
    response
}
