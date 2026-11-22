use super::fixture;
use axum::body::{Body, Bytes};
use axum::http::{Request, StatusCode};
use futures_util::stream;
use http_body_util::BodyExt;
use std::io;
use tokio::sync::mpsc;
use tokio::time::{Duration, timeout};
use tower::ServiceExt;

#[tokio::test]
async fn cancelled_uploads_release_all_permits() {
    let app = fixture::app();
    let mut cancels = Vec::new();
    let mut responses = Vec::new();
    for _ in 0..16 {
        let (tx, rx) = mpsc::channel::<io::Result<Bytes>>(1);
        let body = Body::from_stream(stream::unfold(rx, |mut rx| async move {
            rx.recv().await.map(|item| (item, rx))
        }));
        let res = timeout(
            Duration::from_secs(2),
            app.clone().oneshot(Request::post("/").body(body).unwrap()),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        cancels.push(tx);
        responses.push(res);
    }
    let empty = || Request::post("/").body(Body::empty()).unwrap();
    assert_eq!(
        fixture::status(&app, empty()).await,
        StatusCode::INTERNAL_SERVER_ERROR
    );
    for tx in cancels {
        tx.send(Err(io::Error::new(io::ErrorKind::BrokenPipe, "gone")))
            .await
            .unwrap();
    }
    for res in responses {
        let _ = timeout(Duration::from_secs(2), res.into_body().collect()).await;
    }
    assert_eq!(fixture::status(&app, empty()).await, StatusCode::OK);
}
