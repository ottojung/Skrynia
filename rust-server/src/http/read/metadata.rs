use super::util;
use axum::body::Body;
use axum::http::HeaderValue;
use axum::response::Response;

pub fn set(res: &mut Response<Body>, key: &str, tag: &str) {
    let headers = res.headers_mut();
    headers.insert("content-type", HeaderValue::from_static(util::mime(key)));
    headers.insert("etag", HeaderValue::from_str(tag).unwrap());
    headers.insert("cache-control", HeaderValue::from_static("no-cache"));
    headers.insert("accept-ranges", HeaderValue::from_static("bytes"));
    headers.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
}
