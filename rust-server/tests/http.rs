use axum::body::Body;
use axum::http::{Request, StatusCode};
use skrynia_server::{boot, http};
use std::fs;
use tower::ServiceExt;

fn kernel_with_wasm(name: &str, wasm: &[u8]) -> boot::Kernel {
    let dir = std::env::temp_dir().join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let wasm_path = dir.join("root.wasm");
    let data = dir.join("data");
    fs::create_dir_all(&data).unwrap();
    fs::write(&wasm_path, wasm).unwrap();
    boot::Kernel::start(&wasm_path, &data).unwrap()
}

fn empty_kernel(name: &str) -> boot::Kernel {
    kernel_with_wasm(name, b"\0asm\x01\0\0\0")
}

#[tokio::test]
async fn rejects_non_post_and_other_paths() {
    let app = http::router(empty_kernel("skrynia-v3-http-routes"));
    let req = Request::get("/").body(Body::empty()).unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::METHOD_NOT_ALLOWED);
    let req = Request::post("/other").body(Body::empty()).unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn post_root_requires_valid_abi() {
    let app = http::router(empty_kernel("skrynia-v3-http-post"));
    let req = Request::post("/").body(Body::from("hi")).unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
