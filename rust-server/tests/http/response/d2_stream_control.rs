use super::kernel_with_wasm;
use axum::{
    body::{Body, Bytes},
    http::{HeaderValue, Request, StatusCode, header::TRANSFER_ENCODING},
};
use futures_util::stream;
use http_body_util::BodyExt;
use libskrynia::http;
use std::io;
use tower::ServiceExt;

const ECHO: &str = include_str!("../../../test-fixtures/d2_echo.wat");

#[tokio::test]
async fn normal_chunked_streaming_control() {
    let wasm = wat::parse_str(ECHO).unwrap();
    let app = http::router(kernel_with_wasm("v3-d2-stream-control", &wasm));
    let parts: Vec<Result<Bytes, io::Error>> = vec![
        Ok(Bytes::from_static(b"ab")),
        Ok(Bytes::from_static(b"cde")),
        Ok(Bytes::from_static(b"fg")),
    ];
    let mut r = Request::post("/")
        .body(Body::from_stream(stream::iter(parts)))
        .unwrap();
    r.headers_mut()
        .insert(TRANSFER_ENCODING, HeaderValue::from_static("chunked"));
    let res = app.oneshot(r).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(
        res.into_body().collect().await.unwrap().to_bytes(),
        b"abcdefg"[..]
    );
}
