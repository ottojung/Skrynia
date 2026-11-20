use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use skrynia_server::{boot, http};
use std::fs;
use tower::ServiceExt;
#[tokio::test]
async fn proxy_restart_same_data_dir() {
    let tok = [0x55u8; 32];
    let (k, dir) = super::setup("rst", &super::echo_guest(), &tok);
    drop(k);
    fs::write(dir.join("data/programs/root.wasm"), super::echo_guest()).unwrap();
    let k2 = boot::Kernel::start(&dir.join("root.wasm"), &dir.join("data")).unwrap();
    let app = http::router(k2);
    let res = app
        .oneshot(
            Request::post("/")
                .body(Body::from(super::body(&tok, b"ok")))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(
        res.into_body().collect().await.unwrap().to_bytes().as_ref(),
        b"ok"
    );
    let _ = fs::remove_dir_all(dir);
}
