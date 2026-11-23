use super::{gate, output, output::Chunk, term};
use axum::{
    http::{StatusCode, header::CONTENT_TYPE},
    response::{IntoResponse, Response},
};
use std::io;
use std::sync::atomic::Ordering;
use tokio::sync::mpsc;

fn rejected(t: &term::Term, verdict: &gate::Verdict) -> Option<StatusCode> {
    if verdict.load(Ordering::Relaxed) == 1 {
        return Some(StatusCode::PAYLOAD_TOO_LARGE);
    }
    term::get(t).map(|e| match e.kind() {
        io::ErrorKind::InvalidData => StatusCode::PAYLOAD_TOO_LARGE,
        io::ErrorKind::BrokenPipe | io::ErrorKind::TimedOut => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    })
}

pub async fn response(
    mut receiver: mpsc::Receiver<Chunk>,
    t: term::Term,
    verdict: gate::Verdict,
) -> Response {
    let first = receiver.recv().await;
    if let Some(code) = rejected(&t, &verdict) {
        return code.into_response();
    }
    match first {
        Some(Ok(bytes)) => (
            StatusCode::OK,
            [(CONTENT_TYPE, "application/octet-stream")],
            output::body(bytes, receiver),
        )
            .into_response(),
        Some(Err(e)) if e.kind() == io::ErrorKind::InvalidData => {
            StatusCode::PAYLOAD_TOO_LARGE.into_response()
        }
        Some(Err(e))
            if matches!(
                e.kind(),
                io::ErrorKind::BrokenPipe | io::ErrorKind::TimedOut
            ) =>
        {
            StatusCode::BAD_REQUEST.into_response()
        }
        Some(Err(_)) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        None => StatusCode::OK.into_response(),
    }
}
