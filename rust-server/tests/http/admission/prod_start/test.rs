use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use skrynia_server::{boot, conf, http};
use std::{ffi::OsString, fs};
use tower::ServiceExt;

use super::setup::{body, prepare};

#[tokio::test]
async fn prod_startup_http_separate_roots_no_shadow_and_restart() {
    let dir = std::env::temp_dir().join(format!(
        "sk-http-prod-{}-{}",
        std::process::id(),
        rand::random::<u64>()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let raw = [0x71u8; 32];
    let bad = [0x72u8; 32];
    let (op, user) = prepare(&dir, &raw, &bad);
    let cfg = conf::Config::from_lookup(|k| match k {
        "SKRYNIA_ROOT_WASM" => Some(OsString::from(dir.join("root.wasm"))),
        "SKRYNIA_OPERATOR_DIR" => Some(OsString::from(&op)),
        "SKRYNIA_USER_DIR" => Some(OsString::from(&user)),
        _ => None,
    })
    .unwrap();
    let app = http::router(
        boot::Kernel::start(&cfg.root_wasm, &cfg.operator_dir, &cfg.user_dir).unwrap(),
    );
    super::req::check(app, &raw, &bad).await;
    let k2 = boot::Kernel::start(&cfg.root_wasm, &cfg.operator_dir, &cfg.user_dir).unwrap();
    let ok2 = http::router(k2)
        .oneshot(
            Request::post("/")
                .body(Body::from(body(&raw, b"again")))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(ok2.status(), StatusCode::OK);
    assert_eq!(
        ok2.into_body().collect().await.unwrap().to_bytes().as_ref(),
        b"again"
    );
    let _ = std::fs::remove_dir_all(dir);
}
