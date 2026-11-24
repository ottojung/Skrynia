use crate::kernel_with_wasm;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use libskrynia::http;
use tower::ServiceExt;

fn trap_root() -> Vec<u8> {
    let guest = r#"(module(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 1)(data(i32.const 0) "part")(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(drop(call $w(local.get $out)(i32.const 0)(i32.const 4))) unreachable (i32.const 0)))"#;
    let wasm = wat::parse_str(guest).unwrap();
    let data: String = wasm.iter().map(|b| format!("\\{:02x}", b)).collect();
    let root = format!(
        r#"(module(import "skrynia" "eval"(func $e(param i32 i32 i32 i32 i32)(result i32)))(memory(export "memory") 2)(data(i32.const 0) "{data}")(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(call $e(i32.const 0)(i32.const {len})(local.get $in)(local.get $out)(i32.const -1))))"#,
        data = data,
        len = wasm.len()
    );
    wat::parse_str(root).unwrap()
}

#[tokio::test]
async fn eval_guest_partial_output_then_trap() {
    let app = http::router(kernel_with_wasm("v3-eval-trap", &trap_root()));
    let req = Request::post("/").body(Body::empty()).unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert!(res.into_body().collect().await.is_err());
}
