use super::basic::kernel_with_wasm;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use libskrynia::http;
use tower::ServiceExt;

#[tokio::test]
async fn streaming_output_exceeds_old_one_megabyte_buffer() {
    let wasm = wat::parse_str(include_str!("../../test-fixtures/stream_large_output.wat")).unwrap();
    let app = http::router(kernel_with_wasm("skrynia-large-output", &wasm));
    let request = Request::post("/").body(Body::empty()).unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(body.len(), 2 * 1024 * 1024);
    assert!(body.iter().all(|b| *b == b'L'));
}
