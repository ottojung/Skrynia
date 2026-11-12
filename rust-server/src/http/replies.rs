use axum::body::Body;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};

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
pub fn token(headers: &HeaderMap) -> crate::error::Result<&str> {
    let text = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    let Some(value) = text.strip_prefix("Bearer ") else {
        return Err(crate::error::Fail(
            StatusCode::UNAUTHORIZED,
            "grant_user_token_required",
        ));
    };
    if value.len() != 64
        || !value
            .bytes()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
    {
        return Err(crate::error::Fail(
            StatusCode::UNAUTHORIZED,
            "grant_user_token_required",
        ));
    }
    Ok(value)
}
