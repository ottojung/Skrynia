use super::{flow, input, output, pump, respond, router::App};
use axum::{
    extract::{Request, State},
    http::{Method, StatusCode},
    response::IntoResponse,
};
pub(super) async fn handle(State(app): State<App>, request: Request) -> impl IntoResponse {
    if request.method() != Method::POST || request.uri().path() != "/" {
        return StatusCode::NOT_FOUND.into_response();
    }
    let lim = super::router::limits();
    if request
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<usize>().ok())
        .is_some_and(|n| n > lim.max)
    {
        return StatusCode::PAYLOAD_TOO_LARGE.into_response();
    }
    let permit = match app.permits.try_acquire_owned() {
        Ok(permit) => permit,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    let (tx, rx) = tokio::sync::mpsc::channel(2);
    let (out_tx, out_rx) = tokio::sync::mpsc::channel(2);
    let pump = tokio::spawn(pump::run(request.into_body(), tx, lim));
    let kernel = app.kernel;
    let worker_tx = out_tx.clone();
    let worker = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        kernel.invoke_stream(
            Box::new(input::Input::new(rx)),
            lim.max as u64,
            Box::new(output::Output(worker_tx)),
        )
    });
    tokio::spawn(flow::supervise(pump, worker, out_tx));
    respond::response(out_rx).await
}
