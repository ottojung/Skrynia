use super::basic::kernel_with_wasm;
use axum::{
    body::{Body, Bytes},
    http::{Request, StatusCode},
};
use futures_util::stream;
use http_body_util::BodyExt;
use libskrynia::http;
use tower::ServiceExt;

pub(super) const STREAM_ROOT: &str = r#"(module
  (import "skrynia" "read" (func $read (param i32 i32 i32) (result i32)))
  (import "skrynia" "write" (func $write (param i32 i32 i32) (result i32)))
  (memory (export "memory") 1)
  (func (export "skrynia_main") (param $input i32) (param $output i32) (result i32)
    (local $n i32) (local $done i32)
    (loop $again
      (local.set $n (call $read (local.get $input) (i32.const 4096) (i32.const 4)))
      (if (i32.lt_s (local.get $n) (i32.const 0))
        (then (return (local.get $n))))
      (local.set $done (i32.const 0))
      (block $written
        (loop $write_more
          (br_if $written (i32.eq (local.get $done) (local.get $n)))
          (local.set $done (i32.add (local.get $done)
            (call $write (local.get $output) (i32.add (i32.const 4096) (local.get $done))
              (i32.sub (local.get $n) (local.get $done)))))
          (br_if $write_more (i32.lt_s (local.get $done) (local.get $n)))))
      (br_if $again (i32.gt_s (local.get $n) (i32.const 0))))
    (i32.const 0)))"#;
#[tokio::test]
async fn posted_chunks_arrive_through_descriptor() {
    let wasm = wat::parse_str(STREAM_ROOT).unwrap();
    let app = http::router(kernel_with_wasm("skrynia-http-stream", &wasm));
    let chunks = vec![
        Ok::<Bytes, std::io::Error>(Bytes::from_static(b"ab")),
        Ok(Bytes::from_static(b"cde")),
        Ok(Bytes::from_static(b"fg")),
    ];
    let body = Body::from_stream(stream::iter(chunks));
    let req = Request::post("/").body(body).unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(
        res.into_body().collect().await.unwrap().to_bytes(),
        b"abcdefg"[..]
    );
}
