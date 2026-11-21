use axum::{
    body::{Body, Bytes},
    http::{Request, StatusCode},
};
use futures_util::stream;
use std::io;
use tokio::time::{Duration, advance};
use tower::ServiceExt;

#[tokio::test(start_paused = true)]
async fn full_undrained_channel_times_out() {
    let wasm = wat::parse_str(
        "(module (memory (export \"memory\") 1) (func (export \"skrynia_main\") (param i32 i32) (result i32) (loop $s (br $s)) (i32.const 0)))",
    )
    .unwrap();
    let app = skrynia_server::http::router(super::kernel_with_wasm(
        "skrynia-v3-full-input-timeout",
        &wasm,
    ));
    let body = Body::from_stream(stream::iter(vec![
        Ok::<_, io::Error>(Bytes::from_static(b"a")),
        Ok(Bytes::from_static(b"b")),
        Ok(Bytes::from_static(b"c")),
    ]));
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
