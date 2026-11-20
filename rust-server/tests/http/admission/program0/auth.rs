use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use skrynia_server::http;
use tower::ServiceExt;
#[tokio::test]
#[ignore = "requires compiled fd-eval-adapted skrynia-tools Program0 wasm (SKRYNIA_PROGRAM0_FD_EVAL_WASM)"]
async fn denial_and_token_stripping() {
    let raw = [0x42u8; 32];
    let (k, dir) = super::setup("auth", &super::echo_guest(), &raw);
    let app = http::router(k);
    let ok = app
        .clone()
        .oneshot(
            Request::post("/")
                .body(Body::from(super::body(&raw, b"hi")))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(ok.status(), StatusCode::OK);
    assert_eq!(
        ok.into_body().collect().await.unwrap().to_bytes().as_ref(),
        b"hi"
    );
    let bad = app
        .clone()
        .oneshot(
            Request::post("/")
                .body(Body::from(super::body(&[0x43; 32], b"hi")))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(bad.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let short = app
        .oneshot(Request::post("/").body(Body::from(vec![1, 2, 3])).unwrap())
        .await
        .unwrap();
    assert_eq!(short.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let _ = std::fs::remove_dir_all(dir);
}
