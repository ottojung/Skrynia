use super::kernel_with_wasm;
use axum::{
    body::{Body, Bytes},
    http::{HeaderValue, Request, StatusCode, header::TRANSFER_ENCODING},
};
use futures_util::stream;
use libskrynia::http;
use std::io;
use tower::ServiceExt;

const ECHO: &str = include_str!("../../../test-fixtures/d2_echo.wat");

fn chunked(body: Body) -> Request<Body> {
    let mut r = Request::post("/").body(body).unwrap();
    r.headers_mut()
        .insert(TRANSFER_ENCODING, HeaderValue::from_static("chunked"));
    r
}

#[tokio::test]
async fn oversized_chunked_body_is_rejected_not_truncated_200() {
    let wasm = wat::parse_str(ECHO).unwrap();
    let app = http::router(kernel_with_wasm("v3-d2-chunked", &wasm));
    let parts = vec![Bytes::from(vec![b'Z'; 1 << 20]); 17];
    let body = Body::from_stream(stream::iter(parts.into_iter().map(Ok::<_, io::Error>)));
    let res = app.clone().oneshot(chunked(body)).await.unwrap();
    assert_eq!(res.status(), StatusCode::PAYLOAD_TOO_LARGE);
}

#[tokio::test]
async fn oversize_content_length_control_still_rejected() {
    let wasm = wat::parse_str(ECHO).unwrap();
    let app = http::router(kernel_with_wasm("v3-d2-cl-control", &wasm));
    let req = Request::post("/")
        .header("content-length", 17 * 1024 * 1024)
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::PAYLOAD_TOO_LARGE);
}
