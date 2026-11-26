use axum::{
    body::{Body, Bytes},
    http::{Request, StatusCode},
};
use futures_util::{StreamExt, stream};
use std::io;
use tokio::time::{Duration, advance};
use tower::ServiceExt;

#[tokio::test(start_paused = true)]
async fn full_undrained_channel_times_out() {
    let wasm = wat::parse_str(
        r#"(module
          (import "skrynia" "read1" (func $sk_read1_0 (param i32) (result i32)))(func $read (param $fd i32)(param $dst i32)(param $len i32)(result i32)(local $i i32)(local $b i32)(block $done(loop $lp(br_if $done(i32.ge_u(local.get $i)(local.get $len)))(local.set $b(call $sk_read1_0(local.get $fd)))(if(i32.lt_s(local.get $b)(i32.const 0))(then(if(i32.ne(local.get $b)(i32.const -1))(then(return(local.get $b))))(br $done)))(i32.store8(i32.add(local.get $dst)(local.get $i))(local.get $b))(local.set $i(i32.add(local.get $i)(i32.const 1)))(br $lp)))(local.get $i))
          (memory (export "memory") 1)
          (func (export "skrynia_main") (param i32 i32) (result i32)
            (local $n i32)
            (loop $s
              (local.set $n (call $read (i32.const 0) (i32.const 0) (i32.const 1)))
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
