use axum::{
    body::{Body, Bytes},
    http::{Request, StatusCode},
};
use futures_util::stream;
use http_body_util::BodyExt;
use skrynia_server::{boot, http};
use std::fs;
use tower::ServiceExt;

fn app(name: &str) -> axum::Router {
    let dir = std::env::temp_dir().join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("data")).unwrap();
    let root = r#"(module(import "skrynia" "run_program1"(func $r(param i32 i32)(result i32)))(memory(export "memory") 1)(func(export "skrynia_main")(param $i i32)(param $o i32)(result i32)(call $r(local.get $i)(local.get $o))))"#;
    let p1 = r#"(module(import "skrynia" "read"(func $r(param i32 i32 i32)(result i32)))(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 2)(func(export "skrynia_main")(param $i i32)(param $o i32)(result i32)(local $n i32)(local $d i32)(local $x i32)(block $done(loop $l(local.set $n(call $r(local.get $i)(i32.const 0)(i32.const 4096)))(br_if $done(i32.eq(local.get $n)(i32.const 0)))(if(i32.lt_s(local.get $n)(i32.const 0))(then(return(local.get $n))))(local.set $d(i32.const 0))(block $b(loop $w2(br_if $b(i32.eq(local.get $d)(local.get $n)))(local.set $x(call $w(local.get $o)(local.get $d)(i32.sub(local.get $n)(local.get $d))))(if(i32.lt_s(local.get $x)(i32.const 0))(then(return(local.get $x))))(local.set $d(i32.add(local.get $d)(local.get $x)))(br $w2)))(br $l)))(i32.const 0)))"#;
    fs::write(dir.join("r.wasm"), wat::parse_str(root).unwrap()).unwrap();
    fs::write(dir.join("p1.wasm"), wat::parse_str(p1).unwrap()).unwrap();
    let k = boot::Kernel::start_with(
        &dir.join("r.wasm"),
        &dir.join("data"),
        Some(&dir.join("p1.wasm")),
    )
    .unwrap();
    http::router(k)
}

async fn roundtrip(name: &str, size: usize) {
    let payload: Vec<u8> = (0..size).map(|i| (i % 251) as u8).collect();
    let chunks: Vec<_> = payload
        .chunks(8192)
        .map(|c| Ok::<Bytes, std::io::Error>(Bytes::copy_from_slice(c)))
        .collect();
    let req = Request::post("/")
        .body(Body::from_stream(stream::iter(chunks)))
        .unwrap();
    let res = app(name).oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.into_body().collect().await.unwrap().to_bytes(), payload);
}

#[tokio::test]
async fn chunked_320k_and_4mib_streaming() {
    roundtrip("skrynia-http-p1-320k", 320 * 1024).await;
    roundtrip("skrynia-http-p1-4m", 4 * 1024 * 1024).await;
}
