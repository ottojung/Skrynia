use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use skrynia_server::http;
use std::fs;
use tokio::time::{Duration, timeout};
use tower::ServiceExt;
#[tokio::test]
async fn streams_ten_mebibytes_with_token_stripped() {
    let raw = [0x11u8; 32];
    let (k, dir) = super::setup("str", &super::echo_guest(), &raw);
    let app = http::router(k);
    let payload = vec![b'S'; 10 * 1024 * 1024 + 17];
    let req = Request::post("/")
        .body(Body::from(super::body(&raw, &payload)))
        .unwrap();
    let res = timeout(Duration::from_secs(30), app.oneshot(req))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let got = res.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(got.len(), payload.len());
    assert!(got.iter().all(|b| *b == b'S'));
    let _ = fs::remove_dir_all(dir);
}
