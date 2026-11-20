use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use skrynia_server::http;
use tower::ServiceExt;
#[tokio::test]
async fn trap_partial_then_cross_request_recovery() {
    let tok = [0x77u8; 32];
    let (k, dir) = super::setup("rec", &super::trap_guest(), &tok);
    let app = http::router(k);
    let res = app
        .clone()
        .oneshot(
            Request::post("/")
                .body(Body::from(super::body(&tok, b"x")))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert!(res.into_body().collect().await.is_err());
    let ok = app
        .oneshot(
            Request::post("/")
                .body(Body::from(super::body(&tok, b"x")))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(ok.status(), StatusCode::OK);
    let _ = std::fs::remove_dir_all(dir);
}
