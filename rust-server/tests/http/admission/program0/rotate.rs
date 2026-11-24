use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use libskrynia::http;
use std::fs;
use tower::ServiceExt;
#[tokio::test]
async fn sha256_verifier_rotation_denies_old_token() {
    let raw = [0x42u8; 32];
    let (k, dir) = super::setup("rot", &super::echo_guest(), &raw);
    let app = http::router(k);
    let raw2 = [0x24u8; 32];
    fs::write(
        dir.join("operator/security/program0.sha256"),
        super::verifier(&raw2),
    )
    .unwrap();
    let rot = app
        .clone()
        .oneshot(
            Request::post("/")
                .body(Body::from(super::body(&raw2, b"ok")))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(rot.status(), StatusCode::OK);
    assert_eq!(
        rot.into_body().collect().await.unwrap().to_bytes().as_ref(),
        b"ok"
    );
    let old = app
        .oneshot(
            Request::post("/")
                .body(Body::from(super::body(&raw, b"ok")))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(old.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let _ = fs::remove_dir_all(dir);
}
