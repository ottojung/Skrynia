use super::output::Chunk;
use axum::body::{Body, Bytes};
use axum::http::StatusCode;
use axum::http::header::{CONTENT_TYPE, RETRY_AFTER};
use axum::response::{IntoResponse, Response};
use futures_util::stream;
use std::io;
use tokio::sync::mpsc;

// Bounded, fixed diagnostics: no error strings, tokens, paths or code details.
pub fn error(kind: io::ErrorKind) -> Response {
    let (status, msg) = match kind {
        io::ErrorKind::InvalidData => (StatusCode::PAYLOAD_TOO_LARGE, "payload too large"),
        io::ErrorKind::BrokenPipe | io::ErrorKind::TimedOut => {
            (StatusCode::BAD_REQUEST, "bad request")
        }
        io::ErrorKind::WouldBlock => (StatusCode::SERVICE_UNAVAILABLE, "resource limit"),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, "guest fault"),
    };
    let mut res = (status, [(CONTENT_TYPE, "text/plain")], msg).into_response();
    if status == StatusCode::SERVICE_UNAVAILABLE {
        res.headers_mut().insert(RETRY_AFTER, "1".parse().unwrap());
    }
    res
}

fn body(first: Bytes, receiver: mpsc::Receiver<Chunk>) -> Body {
    let stream = stream::unfold((Some(Ok(first)), receiver), |(first, mut rx)| async move {
        if let Some(chunk) = first {
            return Some((chunk, (None, rx)));
        }
        rx.recv().await.map(|chunk| (chunk, (None, rx)))
    });
    Body::from_stream(stream)
}
pub async fn response(mut receiver: mpsc::Receiver<Chunk>) -> Response {
    match receiver.recv().await {
        Some(Ok(first)) => (
            StatusCode::OK,
            [(CONTENT_TYPE, "application/octet-stream")],
            body(first, receiver),
        )
            .into_response(),
        Some(Err(e)) => error(e.kind()),
        None => (
            StatusCode::OK,
            [(CONTENT_TYPE, "application/octet-stream")],
            Body::empty(),
        )
            .into_response(),
    }
}
