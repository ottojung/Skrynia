use super::fixture;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use tokio::time::{Duration, advance, timeout};
use tower::ServiceExt;

#[tokio::test(start_paused = true)]
async fn stalled_uploads_release_all_permits() {
    let app = fixture::app();
    let mut senders = Vec::new();
    let mut responses = Vec::new();
    for _ in 0..16 {
        let (tx, req) = fixture::input();
        let res = timeout(Duration::from_secs(2), app.clone().oneshot(req))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        senders.push(tx);
        responses.push(res);
    }
    let denied = timeout(
        Duration::from_secs(2),
        app.clone()
            .oneshot(Request::post("/").body(Body::empty()).unwrap()),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(denied.status(), StatusCode::INTERNAL_SERVER_ERROR);
    advance(Duration::from_secs(31)).await;
    for res in responses {
        let _ = res.into_body().collect().await;
    }
    drop(senders);
    let accepted = timeout(
        Duration::from_secs(2),
        app.oneshot(Request::post("/").body(Body::empty()).unwrap()),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(accepted.status(), StatusCode::OK);
}
