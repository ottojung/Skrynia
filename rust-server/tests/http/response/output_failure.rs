use super::kernel_with_wasm;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use skrynia_server::http;
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
