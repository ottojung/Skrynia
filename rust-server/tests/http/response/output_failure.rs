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

#[tokio::test]
async fn output_followed_by_vm_failure_is_not_clean_eof() {
    let wasm = wat::parse_str(include_str!(
        "../../../test-fixtures/stream_failed_output.wat"
    ))
    .unwrap();
    let app = http::router(kernel_with_wasm("skrynia-failed-output", &wasm));
    let request = Request::post("/").body(Body::empty()).unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.into_body().collect().await.is_err());
}

#[tokio::test]
async fn chunked_body_error_after_early_output_is_400_not_200() {
    let wasm = wat::parse_str(include_str!(
        "../../../test-fixtures/stream_early_output.wat"
    ))
    .unwrap();
    let app = http::router(kernel_with_wasm("skrynia-chunked-late-error", &wasm));
    let parts = stream::unfold(0u8, |n| async move {
        match n {
            0 => Some((Ok::<Bytes, io::Error>(Bytes::from_static(b"request")), 1)),
            1 => {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                Some((Err(io::Error::other("late body error")), 2))
            }
            _ => None,
        }
    });
    let mut request = Request::post("/").body(Body::from_stream(parts)).unwrap();
    request
        .headers_mut()
        .insert(TRANSFER_ENCODING, HeaderValue::from_static("chunked"));
    let res = app.oneshot(request).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}
