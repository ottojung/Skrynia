use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use skrynia_server::http;
use tokio::time::{Duration, timeout};
use tower::ServiceExt;
#[tokio::test]
async fn proxy_streams_large_slow_consumer() {
    let tok = [0x11u8; 32];
    let (k, dir) = super::setup("flow", &super::echo_guest(), &tok);
    let app = http::router(k);
    let payload = vec![b'Q'; 10 * 1024 * 1024 + 99];
    let req = Request::post("/")
        .body(Body::from(super::body(&tok, &payload)))
        .unwrap();
    let res = timeout(Duration::from_secs(30), app.oneshot(req))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let mut body = res.into_body();
    let mut got = 0usize;
    while let Some(frame) = body.frame().await {
        let b = frame.unwrap().into_data().unwrap();
        assert!(b.iter().all(|x| *x == b'Q'));
        got += b.len();
        if got <= b.len() {
            tokio::task::yield_now().await;
        }
    }
    assert_eq!(got, payload.len());
    let _ = std::fs::remove_dir_all(dir);
}
