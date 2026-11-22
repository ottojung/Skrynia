use super::{gate, output, output::Chunk, term};
use axum::http::StatusCode;
use axum::http::header::{CONTENT_TYPE, RETRY_AFTER};
use axum::response::{IntoResponse, Response};
use std::io;
use std::sync::atomic::Ordering;
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

fn rejected(t: &term::Term, verdict: &gate::Verdict) -> Option<io::ErrorKind> {
    if verdict.load(Ordering::Relaxed) == 1 {
        return Some(io::ErrorKind::InvalidData);
    }
    term::get(t).map(|e| e.kind())
}

pub async fn response(
    mut receiver: mpsc::Receiver<Chunk>,
    t: term::Term,
    verdict: gate::Verdict,
) -> Response {
    let first = receiver.recv().await;
    if let Some(kind) = rejected(&t, &verdict) {
        return error(kind);
    }
    match first {
        Some(Ok(bytes)) => (
            StatusCode::OK,
            [(CONTENT_TYPE, "application/octet-stream")],
            output::body(bytes, receiver),
        )
            .into_response(),
        Some(Err(e)) => error(e.kind()),
        None => StatusCode::OK.into_response(),
    }
}
