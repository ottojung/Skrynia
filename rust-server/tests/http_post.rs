use axum::body::Body;
use axum::http::{Request, StatusCode};
use skrynia_server::{boot, http};
use std::fs;
use tower::ServiceExt;

#[tokio::test]
async fn post_root_reaches_kernel_path() {
    let dir = std::env::temp_dir().join("skrynia-v3-http-post");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let wasm = dir.join("root.wasm");
    fs::write(&wasm, b"\0asm\x01\0\0\0").unwrap();
    let app = http::router(boot::Kernel::start(&wasm).unwrap());
    let req = Request::post("/").body(Body::from("hi")).unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_IMPLEMENTED);
}
