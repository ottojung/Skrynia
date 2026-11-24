use super::setup::{body, echo_guest, root_wasm};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use libskrynia::{boot, conf, http};
use sha2::{Digest, Sha256};
use std::{ffi::OsString, fs};
use tower::ServiceExt;
#[tokio::test]
async fn missing_operator_verifier_no_user_shadow() {
    let d = std::env::temp_dir().join(format!(
        "sk-pr119-missvf-{}-{}",
        std::process::id(),
        rand::random::<u64>()
    ));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(d.join("op/programs")).unwrap();
    fs::create_dir_all(d.join("op/security")).unwrap();
    fs::create_dir_all(d.join("user/security")).unwrap();
    let raw = [0x55u8; 32];
    fs::write(d.join("user/security/program0.sha256"), Sha256::digest(raw)).unwrap();
    fs::write(d.join("op/programs/root.wasm"), echo_guest()).unwrap();
    fs::write(d.join("root.wasm"), root_wasm()).unwrap();
    let cfg = conf::Config::from_lookup(|k| match k {
        "SKRYNIA_ROOT_WASM" => Some(OsString::from(d.join("root.wasm"))),
        "SKRYNIA_OPERATOR_DIR" => Some(OsString::from(d.join("op"))),
        "SKRYNIA_USER_DIR" => Some(OsString::from(d.join("user"))),
        _ => None,
    })
    .unwrap();
    let app = http::router(
        boot::Kernel::start(&cfg.root_wasm, &cfg.operator_dir, &cfg.user_dir).unwrap(),
    );
    let r = app
        .oneshot(
            Request::post("/")
                .body(Body::from(body(&raw, b"x")))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let _ = fs::remove_dir_all(d);
}
