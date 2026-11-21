use super::setup::body;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use tower::ServiceExt;

pub async fn check(app: axum::Router, raw: &[u8; 32], bad: &[u8; 32]) {
    let denied = app
        .clone()
        .oneshot(
            Request::post("/")
                .body(Body::from(body(bad, b"x")))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(denied.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let ok = app
        .oneshot(
            Request::post("/")
                .body(Body::from(body(raw, b"ok")))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(ok.status(), StatusCode::OK);
    assert_eq!(
        ok.into_body().collect().await.unwrap().to_bytes().as_ref(),
        b"ok"
    );
}
