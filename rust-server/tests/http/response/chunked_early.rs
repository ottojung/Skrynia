use super::kernel_with_wasm;
use axum::{
    body::{Body, Bytes},
    http::{HeaderValue, Request, StatusCode, header::TRANSFER_ENCODING},
};
use futures_util::{StreamExt, stream};
use libskrynia::http;
use std::io;
use tower::ServiceExt;

#[tokio::test]
async fn chunked_early_output_body_is_accepted_after_preflight() {
    let wasm = wat::parse_str(include_str!(
        "../../../test-fixtures/stream_early_output.wat"
    ))
    .unwrap();
    let app = http::router(kernel_with_wasm("skrynia-chunked-early", &wasm));
    let parts: Vec<Result<Bytes, io::Error>> = vec![Ok(Bytes::from_static(b"request"))];
    let mut request = Request::post("/")
        .body(Body::from_stream(stream::iter(parts)))
        .unwrap();
    request
        .headers_mut()
        .insert(TRANSFER_ENCODING, HeaderValue::from_static("chunked"));
    let res = app.oneshot(request).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let mut body = res.into_body().into_data_stream();
    assert_eq!(body.next().await.unwrap().unwrap(), b"A"[..]);
    assert_eq!(body.next().await.unwrap().unwrap(), b"B"[..]);
    assert!(body.next().await.is_none());
}
