use axum::{
    body::{Body, Bytes},
    http::Request,
};
use futures_util::stream;
use skrynia_server::{boot, http};
use std::fs;
use tower::ServiceExt;

#[tokio::test]
async fn client_disconnect_during_input_is_handled() {
    let dir = std::env::temp_dir().join(format!("skrynia-http-p1-cancel-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("data")).unwrap();
    let root = r#"(module(import "skrynia" "run_program1"(func $r(param i32 i32)(result i32)))(memory(export "memory") 1)(func(export "skrynia_main")(param $i i32)(param $o i32)(result i32)(call $r(local.get $i)(local.get $o))))"#;
    let p1 = r#"(module(import "skrynia" "read"(func $r(param i32 i32 i32)(result i32)))(memory(export "memory") 1)(func(export "skrynia_main")(param $i i32)(param $o i32)(result i32)(local $n i32)(block $d(loop $l(local.set $n(call $r(local.get $i)(i32.const 0)(i32.const 64)))(br_if $d(i32.eq(local.get $n)(i32.const 0)))(if(i32.lt_s(local.get $n)(i32.const 0))(then(return(local.get $n))))(br $l)))(i32.const 0)))"#;
    fs::write(dir.join("r.wasm"), wat::parse_str(root).unwrap()).unwrap();
    fs::write(dir.join("p1.wasm"), wat::parse_str(p1).unwrap()).unwrap();
    let k = boot::Kernel::start_with(
        &dir.join("r.wasm"),
        &dir.join("data"),
        Some(&dir.join("p1.wasm")),
    )
    .unwrap();
    let chunks = vec![
        Ok::<Bytes, std::io::Error>(Bytes::from_static(b"abc")),
        Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, "gone")),
    ];
    let req = Request::post("/")
        .body(Body::from_stream(stream::iter(chunks)))
        .unwrap();
    let _ = http::router(k).oneshot(req).await.unwrap();
}
