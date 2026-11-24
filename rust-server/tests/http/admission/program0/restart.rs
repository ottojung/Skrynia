use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use libskrynia::{boot, http};
use std::fs;
use tower::ServiceExt;
#[tokio::test]
async fn restart_preserves_rotated_sha256_verifier() {
    let raw = [0x24u8; 32];
    let (k, dir) = super::setup("rst", &super::echo_guest(), &raw);
    drop(k);
    fs::write(
        dir.join("operator/security/program0.sha256"),
        super::verifier(&raw),
    )
    .unwrap();
    let k2 = boot::Kernel::start_with_roots(
        &dir.join("root.wasm"),
        &dir.join("operator"),
        &dir.join("user"),
    )
    .unwrap();
    let ok = http::router(k2)
        .oneshot(
            Request::post("/")
                .body(Body::from(super::body(&raw, b"r")))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(ok.status(), StatusCode::OK);
    assert_eq!(
        ok.into_body().collect().await.unwrap().to_bytes().as_ref(),
        b"r"
    );
    let _ = fs::remove_dir_all(dir);
}
