use super::fixture;
use axum::{
    body::{Body, Bytes},
    http::{Request, StatusCode},
};
use futures_util::stream;
use http_body_util::BodyExt;
use std::io;
use tokio::time::{Duration, advance, timeout};
use tower::ServiceExt;

#[tokio::test(start_paused = true)]
async fn cancelled_upload_releases_permit() {
    let app = fixture::app();
    let chunks = vec![
        Ok::<Bytes, io::Error>(Bytes::from_static(b"x")),
        Err(io::Error::new(io::ErrorKind::BrokenPipe, "gone")),
    ];
    let req = Request::post("/")
        .body(Body::from_stream(stream::iter(chunks)))
        .unwrap();
    let res = timeout(Duration::from_secs(2), app.clone().oneshot(req))
        .await
        .unwrap()
        .unwrap();
    let _ = timeout(Duration::from_secs(2), res.into_body().collect()).await;
    advance(Duration::from_secs(1)).await;
    let ok = timeout(
        Duration::from_secs(2),
        app.oneshot(Request::post("/").body(Body::empty()).unwrap()),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(ok.status(), StatusCode::OK);
}
