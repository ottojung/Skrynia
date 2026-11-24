use super::{basic::kernel_with_wasm, streaming::STREAM_ROOT};
use axum::{
    body::{Body, Bytes},
    http::{Request, StatusCode},
};
use futures_util::stream;
use libskrynia::http;
use tower::ServiceExt;
#[tokio::test]
async fn streaming_transport_failure_is_not_eof() {
    let wasm = wat::parse_str(STREAM_ROOT).unwrap();
    let app = http::router(kernel_with_wasm("skrynia-http-stream-fail", &wasm));
    let chunks = vec![
        Ok::<Bytes, std::io::Error>(Bytes::from_static(b"a")),
        Err(std::io::ErrorKind::BrokenPipe.into()),
    ];
    let req = Request::post("/")
        .body(Body::from_stream(stream::iter(chunks)))
        .unwrap();
    assert_eq!(
        app.oneshot(req).await.unwrap().status(),
        StatusCode::BAD_REQUEST
    );
}
