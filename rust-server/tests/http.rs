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
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
    let req = Request::post("/other").body(Body::empty()).unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn post_root_requires_valid_abi() {
    let app = http::router(empty_kernel("skrynia-v3-http-post"));
    let req = Request::post("/").body(Body::from("hi")).unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[test]
fn nested_eval_callback() {
    let root = r#"(module(import "skrynia" "eval"(func $e(param i32 i32 i32 i32 i32 i32 i32)(result i32)))(table(export "__indirect_function_table") 1 funcref)(memory(export "memory") 3)(func $cb(param i32 i32 i32 i32)(result i32)(i32.store8(local.get 2)(i32.const 75))(i32.const 1))(elem(i32.const 0) func $cb)(func(export "skrynia_main")(param i32 i32 i32 i32)(result i32)(call $e(local.get 0)(local.get 1)(i32.const 65536)(i32.const 0)(i32.const 0)(local.get 2)(local.get 3))))"#;
    let child = r#"(module(import "skrynia" "invoke"(func $i(param i32 i32 i32 i32)(result i32)))(memory(export "memory") 1)(func(export "skrynia_main")(param i32 i32 i32 i32)(result i32)(call $i(i32.const 0)(i32.const 0)(local.get 2)(local.get 3))))"#;
    let dir = std::env::temp_dir().join("skrynia-v3-nested");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("data")).unwrap();
    std::fs::write(dir.join("r.wasm"), wat::parse_str(root).unwrap()).unwrap();
    let k = skrynia_server::boot::Kernel::start(&dir.join("r.wasm"), &dir.join("data")).unwrap();
    assert_eq!(k.invoke(&wat::parse_str(child).unwrap()).unwrap(), b"K");
}
