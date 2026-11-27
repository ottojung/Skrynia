use super::kernel_with_wasm;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use libskrynia::http;
use std::time::Duration;
use tower::ServiceExt;

const BIG: &str = r#"(module
  (import "skrynia" "write1" (func $sk_write1_0 (param i32 i32) (result i32)))(func $w (param $fd i32)(param $src i32)(param $len i32)(result i32)(local $i i32)(local $r i32)(block $done(loop $lp(br_if $done(i32.ge_u(local.get $i)(local.get $len)))(local.set $r(call $sk_write1_0(local.get $fd)(i32.load8_u(i32.add(local.get $src)(local.get $i)))))(if(i32.lt_s(local.get $r)(i32.const 0))(then(return(local.get $r))))(local.set $i(i32.add(local.get $i)(i32.const 1)))(br $lp)))(local.get $i))
  (memory (export "memory") 2)
  (data (i32.const 0) "L")
  (func (export "skrynia_main") (param i32 i32) (result i32)
    (local $i i32)
    (memory.fill (i32.const 0) (i32.const 76) (i32.const 16384))
    (loop $a
      (drop (call $w (local.get 1) (i32.const 0) (i32.const 16384)))
      (local.set $i (i32.add (local.get $i) (i32.const 1)))
      (br_if $a (i32.lt_u (local.get $i) (i32.const 832))))
    (i32.const 0)))"#;

#[tokio::test]
async fn unread_responses_do_not_starve_admission_forever() {
    let wasm = wat::parse_str(BIG).unwrap();
    let app = http::router(kernel_with_wasm("v3-d1-permit", &wasm));
    let mut held = Vec::new();
    for _ in 0..16 {
        let req = Request::post("/").body(Body::from("x")).unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        held.push(res);
    }
    tokio::time::sleep(Duration::from_millis(100)).await;
    let denied = app
        .clone()
        .oneshot(Request::post("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(denied.status(), StatusCode::SERVICE_UNAVAILABLE);
    drop(held);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let ok = app
        .oneshot(Request::post("/").body(Body::from("x")).unwrap())
        .await
        .unwrap();
    assert_eq!(ok.status(), StatusCode::OK);
}
