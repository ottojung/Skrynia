use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use skrynia_server::http;
use tokio::time::{Duration, timeout};
use tower::ServiceExt;
#[tokio::test]
async fn slow_consumer_streams_large_payload() {
    let raw = [0x71u8; 32];
    let (k, dir) = super::setup("slow", &super::echo_guest(), &raw);
    let app = http::router(k);
    let payload = vec![b'Q'; 10 * 1024 * 1024 + 33];
    let req = Request::post("/")
        .body(Body::from(super::body(&raw, &payload)))
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
