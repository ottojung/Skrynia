use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use skrynia_server::{boot, http};
use std::fs;
use tower::ServiceExt;

#[tokio::test]
async fn trap_after_partial_output_fails_response() {
    let dir = std::env::temp_dir().join(format!("skrynia-http-p1-trap-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("data")).unwrap();
    let root = r#"(module(import "skrynia" "run_program1"(func $r(param i32 i32)(result i32)))(memory(export "memory") 1)(func(export "skrynia_main")(param $i i32)(param $o i32)(result i32)(call $r(local.get $i)(local.get $o))))"#;
    let p1 = r#"(module(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 1)(data(i32.const 0) "ab")(func(export "skrynia_main")(param $i i32)(param $o i32)(result i32)(drop(call $w(local.get $o)(i32.const 0)(i32.const 2)))(unreachable)(i32.const 0)))"#;
    fs::write(dir.join("r.wasm"), wat::parse_str(root).unwrap()).unwrap();
    fs::write(dir.join("p1.wasm"), wat::parse_str(p1).unwrap()).unwrap();
    let k = boot::Kernel::start_with(
        &dir.join("r.wasm"),
        &dir.join("data"),
        Some(&dir.join("p1.wasm")),
    )
    .unwrap();
    let req = Request::post("/").body(Body::from("x")).unwrap();
    let res = http::router(k).oneshot(req).await.unwrap();
    let ok = res.status() == StatusCode::OK;
    let body = res.into_body().collect().await;
    assert!(!ok || body.is_err());
}
