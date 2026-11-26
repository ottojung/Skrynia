use super::{flow, gate, input, output, pump, respond, router::App, term};
use axum::{
    extract::{Request, State},
    http::{Method, StatusCode},
    response::IntoResponse,
};
use std::sync::Arc;
use std::sync::atomic::AtomicU8;
use tokio::sync::mpsc;

pub(super) async fn handle(State(app): State<App>, request: Request) -> impl IntoResponse {
    if request.method() != Method::POST || request.uri().path() != "/" {
        return StatusCode::NOT_FOUND.into_response();
    }
    let lim = super::router::limits();
    if gate::declared_too_large(&request, lim.max) {
        return StatusCode::PAYLOAD_TOO_LARGE.into_response();
    }
    let permit = match app.permits.try_acquire_owned() {
        Ok(permit) => permit,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let chunked = gate::chunked(&request);
    let t = term::new();
    let verdict: gate::Verdict = Arc::new(AtomicU8::new(0));
    let (tx, rx) = mpsc::channel(2);
    let (out_tx, out_rx) = mpsc::channel(2);
    let deadline = tokio::time::Instant::now() + super::router::response_total();
    let body = if chunked {
        gate::body(request.into_body(), lim.max, verdict.clone())
    } else {
        request.into_body()
    };
    let pump = tokio::spawn(pump::run(body, tx, t.clone(), lim));
    let kernel = app.kernel;
    let worker_tx = out_tx.clone();
    let input_term = t.clone();
    let worker = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        kernel.invoke_stream(
            Box::new(input::Input::new(rx, input_term)),
            lim.max as u64,
            Box::new(output::Output(worker_tx, deadline)),
        )
    });
    tokio::spawn(flow::supervise(pump, worker, out_tx));
    respond::response(out_rx, t, verdict).await
}
