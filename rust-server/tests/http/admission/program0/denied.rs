use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use libskrynia::http;
use tower::ServiceExt;
fn privileged_guest() -> Vec<u8> {
    wat::parse_str(r#"(module(import "skrynia" "open_disk"(func $od(param i64 i64 i32)(result i32)))(memory(export "memory") 1)(func(export "skrynia_main")(param $i i32)(param $o i32)(result i32)(call $od (i64.const 0) (i64.const 0) (i32.const 1))))"#).unwrap()
}
fn trap_guest() -> Vec<u8> {
    wat::parse_str(r#"(module(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 1)(data(i32.const 0)"part")(func(export "skrynia_main")(param $i i32)(param $o i32)(result i32)(drop(call $w(local.get $o)(i32.const 0)(i32.const 4))) unreachable (i32.const 0)))"#).unwrap()
}
#[tokio::test]
async fn denied_guest_imports_and_traps() {
    let raw = [0x31u8; 32];
    let (k, dir) = super::setup("deny", &privileged_guest(), &raw);
    let app = http::router(k);
    let res = app
        .clone()
        .oneshot(
            Request::post("/")
                .body(Body::from(super::body(&raw, b"x")))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
    drop(app);
    let _ = std::fs::remove_dir_all(&dir);
    let raw = [0x32u8; 32];
    let (k, dir) = super::setup("trap", &trap_guest(), &raw);
    let app = http::router(k);
    let res = app
        .oneshot(
            Request::post("/")
                .body(Body::from(super::body(&raw, b"x")))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert!(res.into_body().collect().await.is_err());
    let _ = std::fs::remove_dir_all(dir);
}
