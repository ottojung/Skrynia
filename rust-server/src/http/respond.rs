use super::output::Chunk;
use axum::{
    body::{Body, Bytes},
    http::{StatusCode, header::CONTENT_TYPE},
    response::{IntoResponse, Response},
};
use futures_util::stream;
use std::io;
use tokio::sync::mpsc;

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
        Some(Err(e)) if e.kind() == io::ErrorKind::InvalidData => {
            StatusCode::PAYLOAD_TOO_LARGE.into_response()
        }
        Some(Err(e)) if e.kind() == io::ErrorKind::BrokenPipe => {
            StatusCode::BAD_REQUEST.into_response()
        }
        Some(Err(_)) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        None => (
            StatusCode::OK,
            [(CONTENT_TYPE, "application/octet-stream")],
            Body::empty(),
        )
            .into_response(),
    }
}
