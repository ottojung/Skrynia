use axum::{
    body::{Body, Bytes},
    http::{Request, StatusCode},
};
use futures_util::stream;
use skrynia_server::http;
use std::io;
use tokio::time::{Duration, timeout};
use tower::ServiceExt;
#[tokio::test]
async fn proxy_cancelled_upload_recovers() {
    let tok = [0x66u8; 32];
    let (k, dir) = super::setup("can", &super::echo_guest(), &tok);
    let app = http::router(k);
    let chunks = vec![
        Ok::<Bytes, io::Error>(Bytes::copy_from_slice(&super::body(&tok, b"ab"))),
        Err(io::Error::new(io::ErrorKind::BrokenPipe, "gone")),
    ];
    let req = Request::post("/")
        .body(Body::from_stream(stream::iter(chunks)))
        .unwrap();
    let res = timeout(Duration::from_secs(5), app.clone().oneshot(req))
        .await
        .unwrap()
        .unwrap();
    assert!(matches!(
        res.status(),
        StatusCode::BAD_REQUEST | StatusCode::INTERNAL_SERVER_ERROR | StatusCode::OK
    ));
    let ok = app
        .oneshot(
            Request::post("/")
                .body(Body::from(super::body(&tok, b"z")))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(ok.status(), StatusCode::OK);
    let _ = std::fs::remove_dir_all(dir);
}
