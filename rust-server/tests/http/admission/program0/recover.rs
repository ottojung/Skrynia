use axum::{
    body::{Body, Bytes},
    http::{Request, StatusCode},
};
use futures_util::stream;
use libskrynia::{boot, http};
use std::{fs, io};
use tokio::time::{Duration, timeout};
use tower::ServiceExt;
#[tokio::test]
async fn cancellation_and_restart() {
    let raw = [0x55u8; 32];
    let (k, dir) = super::setup("rec", &super::echo_guest(), &raw);
    let app = http::router(k);
    let chunks = vec![
        Ok::<Bytes, io::Error>(Bytes::copy_from_slice(&super::body(&raw, b"ab"))),
        Err(io::Error::new(io::ErrorKind::BrokenPipe, "gone")),
    ];
    let req = Request::post("/")
        .body(Body::from_stream(stream::iter(chunks)))
        .unwrap();
    let _ = timeout(Duration::from_secs(5), app.clone().oneshot(req))
        .await
        .unwrap();
    let ok = app
        .oneshot(
            Request::post("/")
                .body(Body::from(super::body(&raw, b"z")))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(ok.status(), StatusCode::OK);
    drop(ok);
    let root = dir.join("root.wasm");
    let op = dir.join("operator");
    let usr = dir.join("user");
    let k2 = boot::Kernel::start_with_roots(&root, &op, &usr).unwrap();
    let ok = http::router(k2)
        .oneshot(
            Request::post("/")
                .body(Body::from(super::body(&raw, b"r")))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(ok.status(), StatusCode::OK);
    let _ = std::fs::remove_dir_all(dir);
}
