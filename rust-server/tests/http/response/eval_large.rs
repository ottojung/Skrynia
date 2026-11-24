use crate::kernel_with_wasm;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use libskrynia::http;
use tower::ServiceExt;

fn eval_echo_root() -> Vec<u8> {
    let guest = r#"(module(import "skrynia" "read"(func $r(param i32 i32 i32)(result i32)))(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 2)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $n i32)(local $d i32)(local $w i32)(block $done(loop $lp(local.set $n(call $r(local.get $in)(i32.const 0)(i32.const 65536)))(br_if $done(i32.eqz(local.get $n)))(if(i32.lt_s(local.get $n)(i32.const 0))(then(return(local.get $n))))(local.set $d(i32.const 0))(block $wok(loop $wl(br_if $wok(i32.ge_s(local.get $d)(local.get $n)))(local.set $w(call $w(local.get $out)(local.get $d)(i32.sub(local.get $n)(local.get $d))))(if(i32.lt_s(local.get $w)(i32.const 0))(then(return(local.get $w))))(local.set $d(i32.add(local.get $d)(local.get $w)))(br $wl)))(br $lp)))(i32.const 0)))"#;
    let wasm = wat::parse_str(guest).unwrap();
    let data: String = wasm.iter().map(|b| format!("\\{:02x}", b)).collect();
    let root = format!(
        r#"(module(import "skrynia" "eval"(func $e(param i32 i32 i32 i32 i32)(result i32)))(memory(export "memory") 4)(data(i32.const 0) "{data}")(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(call $e(i32.const 0)(i32.const {len})(local.get $in)(local.get $out)(i32.const -1))))"#,
        data = data,
        len = wasm.len()
    );
    wat::parse_str(root).unwrap()
}

#[tokio::test]
async fn eval_guest_streams_over_ten_mebibytes() {
    let app = http::router(kernel_with_wasm("v3-eval-10m", &eval_echo_root()));
    let size = 10 * 1024 * 1024 + 321;
    let body = vec![b'Z'; size];
    let req = Request::post("/").body(Body::from(body.clone())).unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let got = res.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(got.len(), size);
    assert!(got.iter().all(|b| *b == b'Z'));
}
