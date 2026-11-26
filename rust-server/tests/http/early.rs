use super::basic::kernel_with_wasm;
use axum::{
    body::{Body, Bytes},
    http::{Request, StatusCode},
};
use futures_util::stream;
use libskrynia::http;
use tower::ServiceExt;
#[tokio::test]
async fn root_can_return_without_consuming_endless_input() {
    let code = wat::parse_str(
        r#"(module
        (import "skrynia" "write1" (func $sk_write1_0 (param i32 i32) (result i32)))(func $w (param $fd i32)(param $src i32)(param $len i32)(result i32)(local $i i32)(local $r i32)(block $done(loop $lp(br_if $done(i32.ge_u(local.get $i)(local.get $len)))(local.set $r(call $sk_write1_0(local.get $fd)(i32.load8_u(i32.add(local.get $src)(local.get $i)))))(if(i32.lt_s(local.get $r)(i32.const 0))(then(return(local.get $r))))(local.set $i(i32.add(local.get $i)(i32.const 1)))(br $lp)))(local.get $i))
        (memory(export "memory") 1) (data (i32.const 8) "X")
        (func(export "skrynia_main")(param i32 i32)(result i32)
          (drop (call $w (local.get 1) (i32.const 8) (i32.const 1)))
          (i32.const 0)))"#,
    )
    .unwrap();
    let app = http::router(kernel_with_wasm("skrynia-root-early", &code));
    let stream = stream::pending::<Result<Bytes, std::io::Error>>();
    let req = Request::post("/").body(Body::from_stream(stream)).unwrap();
    let out = tokio::time::timeout(std::time::Duration::from_secs(2), app.oneshot(req))
        .await
        .expect("root must cancel unread HTTP body")
        .unwrap();
    assert_eq!(out.status(), StatusCode::OK);
}
