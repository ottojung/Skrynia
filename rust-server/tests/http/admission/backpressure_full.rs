use axum::{
    body::{Body, Bytes},
    http::{Request, StatusCode},
};
use futures_util::stream;
use http_body_util::BodyExt;
use tokio::time::{Duration, advance};
use tower::ServiceExt;

const WAT: &str = "(module (import \"skrynia\" \"write\" (func $w (param i32 i32 i32) (result i32))) (import \"skrynia\" \"read\" (func $r (param i32 i32 i32) (result i32))) (memory (export \"memory\") 1) (data (i32.const 0) \"AZ\") (func (export \"skrynia_main\") (param $in i32) (param $out i32) (result i32) (local $n i32) (local $j i32) (loop $wr (drop (call $w (local.get $out) (i32.const 0) (i32.const 1))) (local.set $j (i32.add (local.get $j) (i32.const 1))) (br_if $wr (i32.lt_u (local.get $j) (i32.const 40)))) (loop $rd (local.set $n (call $r (local.get $in) (i32.const 256) (i32.const 1))) (br_if $rd (i32.gt_s (local.get $n) (i32.const 0)))) (if (i32.eqz (local.get $n)) (then (drop (call $w (local.get $out) (i32.const 1) (i32.const 1))))) (i32.const 0)))";

#[tokio::test(start_paused = true)]
async fn full_undrained_channel_has_no_false_eof() {
    let app = skrynia_server::http::router(super::kernel_with_wasm(
        "skrynia-v3-full-channel-eof",
        wat::parse_str(WAT).unwrap().as_slice(),
    ));
    let chunks = (0..8).map(|_| Ok::<Bytes, std::io::Error>(Bytes::from_static(b"x")));
    let req = Request::post("/")
        .body(Body::from_stream(stream::iter(chunks)))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    tokio::task::yield_now().await;
    advance(Duration::from_secs(301)).await;
    let body = tokio::select! {
        b = res.into_body().collect() => b,
        _ = tokio::task::spawn_blocking(|| std::thread::sleep(std::time::Duration::from_millis(900))) =>
            panic!("stalled full-channel upload did not terminate"),
    };
    match body {
        Err(e) => assert!(
            e.to_string().contains("stalled request body"),
            "terminal kind"
        ),
        Ok(b) => assert!(!b.to_bytes().contains(&b'Z'), "no clean EOF"),
    }
    let ok = app
        .oneshot(Request::post("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(ok.status(), StatusCode::OK, "permit released after timeout");
}
