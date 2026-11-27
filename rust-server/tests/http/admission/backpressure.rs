use axum::{
    body::{Body, Bytes},
    http::{Request, StatusCode},
};
use futures_util::{StreamExt, stream};
use std::io;
use tokio::time::{Duration, advance};
use tower::ServiceExt;

#[tokio::test(start_paused = true)]
async fn incomplete_input_times_out() {
    let wasm = wat::parse_str(
        r#"(module
          (import "skrynia" "read1" (func $read (param i32) (result i32)))
          (memory (export "memory") 1)
          (func (export "skrynia_main") (param $in i32) (param i32) (result i32)
            (local $n i32)
            (loop $s
              (local.set $n (call $read (local.get $in)))
              (br_if $s (i32.ge_s (local.get $n) (i32.const 0))))
            (i32.const 0)))"#,
    )
    .unwrap();
    let app = libskrynia::http::router(super::kernel_with_wasm(
        "skrynia-v3-full-input-timeout",
        &wasm,
    ));
    let body = Body::from_stream(
        stream::iter(vec![
            Ok::<_, io::Error>(Bytes::from_static(b"a")),
            Ok(Bytes::from_static(b"b")),
            Ok(Bytes::from_static(b"c")),
        ])
        .chain(stream::pending()),
    );
    let req = Request::post("/").body(body).unwrap();
    let res = tokio::spawn(app.oneshot(req));
    for _ in 0..4 {
        tokio::task::yield_now().await;
    }
    advance(Duration::from_secs(301)).await;
    for _ in 0..4 {
        tokio::task::yield_now().await;
    }
    let res = res.await.unwrap().unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}
