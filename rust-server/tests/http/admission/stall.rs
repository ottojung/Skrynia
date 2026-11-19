use super::fixture;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use tokio::time::{Duration, advance, timeout};
use tower::ServiceExt;

#[tokio::test(start_paused = true)]
async fn stalled_upload_times_out_without_false_eof() {
    let app = fixture::app();
    let (tx, req) = fixture::input();
    let res = timeout(Duration::from_secs(2), app.clone().oneshot(req))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    advance(Duration::from_secs(31)).await;
    tokio::task::yield_now().await;
    let body = timeout(Duration::from_secs(2), res.into_body().collect())
        .await
        .unwrap();
    assert!(body.is_err() || body.unwrap().to_bytes() != b"AB"[..]);
    drop(tx);
    let ok = timeout(
        Duration::from_secs(2),
        app.oneshot(Request::post("/").body(Body::empty()).unwrap()),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(ok.status(), StatusCode::OK);
}
