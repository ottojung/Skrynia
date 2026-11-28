use super::{Common, preflight_fixture, run};
use axum::{
    body::{Body, Bytes},
    http::{HeaderValue, Request, StatusCode, header::TRANSFER_ENCODING},
};
use futures_util::stream;
use std::io;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::Duration;

#[tokio::test]
async fn preflight_uses_request_total_not_response_deadline() {
    let (out_tx, out_rx) = mpsc::channel(4);
    let common = Common {
        kernel: preflight_fixture::kernel(),
        lim: super::super::pump::Limits {
            max: 1024,
            inactivity: Duration::from_secs(5),
            total: Duration::from_millis(1),
        },
        t: super::super::term::new(),
        out_tx,
        out_rx,
        deadline: tokio::time::Instant::now() + Duration::from_secs(100),
        terminal: {
            let (tx, _rx) = mpsc::channel(4);
            tx.clone().try_reserve_owned().unwrap()
        },
    };
    let parts = stream::pending::<Result<Bytes, io::Error>>();
    let mut request = Request::post("/").body(Body::from_stream(parts)).unwrap();
    request
        .headers_mut()
        .insert(TRANSFER_ENCODING, HeaderValue::from_static("chunked"));
    let permit = Arc::new(tokio::sync::Semaphore::new(1))
        .try_acquire_owned()
        .unwrap();
    let res = tokio::time::timeout(Duration::from_secs(2), run(common, request, permit))
        .await
        .expect("preflight must stop at request total, not response deadline");
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}
