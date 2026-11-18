use axum::body::Body;
use axum::http::{Request, StatusCode};
use skrynia_server::{boot, http};
use std::fs;
use tower::ServiceExt;

fn kernel() -> boot::Kernel {
    let dir = std::env::temp_dir().join("skrynia-v3-http");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let wasm = dir.join("root.wasm");
    let data = dir.join("data");
    fs::create_dir_all(&data).unwrap();
    fs::write(&wasm, b"\0asm\x01\0\0\0").unwrap();
    boot::Kernel::start(&wasm, &data).unwrap()
}

#[tokio::test]
async fn rejects_non_post_and_other_paths() {
    let app = http::router(kernel());
    let req = Request::get("/").body(Body::empty()).unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::METHOD_NOT_ALLOWED);
    let req = Request::post("/other").body(Body::empty()).unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::METHOD_NOT_ALLOWED);
}
