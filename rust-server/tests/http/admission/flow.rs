use super::fixture;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use tokio::time::{Duration, timeout};
use tower::ServiceExt;
#[tokio::test]
async fn active_vm_execution_cannot_exceed_admission_limit() {
    let app = fixture::app();
    let mut senders = Vec::new();
    let mut responses = Vec::new();
    for _ in 0..16 {
        let (tx, req) = fixture::input();
        let response = timeout(Duration::from_secs(2), app.clone().oneshot(req))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        senders.push(tx);
        responses.push(response);
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
    drop(senders);
    for response in responses {
        response.into_body().collect().await.unwrap();
    }
    let accepted = timeout(
        Duration::from_secs(2),
        app.oneshot(Request::post("/").body(Body::empty()).unwrap()),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(accepted.status(), StatusCode::OK);
}
