mod collect;
mod preflight;
mod reject;
mod stream;
use super::{flow, gate, input, output, pump, respond, router::App, term};
use axum::{
    extract::{Request, State},
    http::{Method, StatusCode},
    response::IntoResponse,
};
use std::sync::Arc;
use tokio::sync::mpsc;

pub(super) struct Common {
    kernel: Arc<crate::boot::Kernel>,
    lim: pump::Limits,
    t: term::Term,
    out_tx: mpsc::Sender<output::Chunk>,
    out_rx: mpsc::Receiver<output::Chunk>,
    deadline: tokio::time::Instant,
    terminal: mpsc::OwnedPermit<output::Chunk>,
}

pub(super) async fn handle(State(app): State<App>, request: Request) -> impl IntoResponse {
    if request.method() != Method::POST || request.uri().path() != "/" {
        return StatusCode::NOT_FOUND.into_response();
    }
    let lim = super::router::limits();
    if gate::declared_too_large(&request, lim.max) {
        return StatusCode::PAYLOAD_TOO_LARGE.into_response();
    }
    let permit = match app.permits.try_acquire_owned() {
        Ok(p) => p,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let (out_tx, out_rx) = mpsc::channel(output::DATA_SLOTS + 1);
    let common = Common {
        kernel: app.kernel,
        lim,
        t: term::new(),
        terminal: out_tx.clone().try_reserve_owned().expect("fresh queue"),
        out_tx,
        out_rx,
        deadline: tokio::time::Instant::now() + super::router::response_total(),
    };
    if gate::chunked(&request) {
        preflight::run(common, request, permit).await
    } else {
        stream::run(common, request, permit).await
    }
}
