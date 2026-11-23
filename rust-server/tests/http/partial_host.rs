use super::basic::kernel_with_wasm;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use skrynia_server::http;
use tower::ServiceExt;

#[tokio::test]
async fn large_host_io_is_short_and_exact() {
    let code = wat::parse_str(include_str!("../../test-fixtures/stream_partial_host.wat")).unwrap();
    let app = http::router(kernel_with_wasm("skrynia-partial-host-io", &code));
    let bytes = vec![0xA5; 131072];
    let request = Request::post("/").body(Body::from(bytes.clone())).unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.into_body().collect().await.unwrap().to_bytes(),
        bytes
    );
}
