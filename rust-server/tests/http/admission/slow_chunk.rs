use axum::{
    body::{Body, Bytes},
    http::{HeaderValue, Request, StatusCode, header::TRANSFER_ENCODING},
};
use futures_util::stream;
use http_body_util::BodyExt;
use libskrynia::http;
use std::io;
use tokio::time::Duration;
use tower::ServiceExt;

const ECHO: &str = include_str!("../../../test-fixtures/d2_echo.wat");

fn paced(parts: &'static [&'static [u8]], gap: Duration) -> Body {
    Body::from_stream(stream::unfold(
        (parts, 0usize),
        move |(parts, i)| async move {
            if i >= parts.len() {
                return None;
            }
            tokio::time::sleep(gap).await;
            Some((
                Ok::<_, io::Error>(Bytes::from_static(parts[i])),
                (parts, i + 1),
            ))
        },
    ))
}

#[tokio::test(start_paused = true)]
async fn active_slow_chunked_upload_is_not_a_stall() {
    let wasm = wat::parse_str(ECHO).unwrap();
    let app = http::router(super::kernel_with_wasm("skrynia-http-slow-chunk", &wasm));
    let parts: &'static [&'static [u8]] = &[b"a", b"b", b"c", b"d"];
    let mut req = Request::post("/")
        .body(paced(parts, Duration::from_secs(10)))
        .unwrap();
    req.headers_mut()
        .insert(TRANSFER_ENCODING, HeaderValue::from_static("chunked"));
    let task = tokio::spawn(app.oneshot(req));
    super::tick(Duration::from_secs(10), 6).await;
    let res = task.await.unwrap().unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(
        res.into_body().collect().await.unwrap().to_bytes(),
        b"abcd"[..]
    );
}
