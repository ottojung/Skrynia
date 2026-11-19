use super::basic::kernel_with_wasm;
use axum::{
    body::{Body, Bytes},
    http::{Request, StatusCode},
};
use futures_util::{StreamExt, stream};
use skrynia_server::http;
use tower::ServiceExt;

#[tokio::test]
async fn response_is_available_before_request_eof() {
    let wasm = wat::parse_str(include_str!("../../test-fixtures/stream_early_output.wat")).unwrap();
    let app = http::router(kernel_with_wasm("skrynia-first-output", &wasm));
    let (tx, rx) = tokio::sync::mpsc::channel::<Bytes>(2);
    let input = stream::unfold(rx, |mut r| async move {
        r.recv().await.map(|b| (Ok::<_, std::io::Error>(b), r))
    });
    let request = Request::post("/").body(Body::from_stream(input)).unwrap();
    let res = tokio::time::timeout(std::time::Duration::from_secs(2), app.oneshot(request))
        .await
        .expect("output must not await request EOF")
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let mut body = res.into_body().into_data_stream();
    assert_eq!(body.next().await.unwrap().unwrap(), b"A"[..]);
    tx.send(Bytes::from_static(b"request")).await.unwrap();
    drop(tx);
    assert_eq!(body.next().await.unwrap().unwrap(), b"B"[..]);
    assert!(body.next().await.is_none());
}
