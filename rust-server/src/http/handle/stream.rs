use super::{Common, flow, input, output, pump, respond};
use axum::{extract::Request, response::Response};
use tokio::sync::{OwnedSemaphorePermit, mpsc};

pub(super) async fn run(
    common: Common,
    request: Request,
    permit: OwnedSemaphorePermit,
) -> Response {
    let Common {
        kernel,
        lim,
        t,
        verdict,
        out_tx,
        out_rx,
        deadline,
        terminal,
    } = common;
    let (tx, rx) = mpsc::channel(2);
    let pump = tokio::spawn(pump::run(request.into_body(), tx, t.clone(), lim, false));
    let worker_term = t.clone();
    let worker = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        kernel.invoke_stream(
            Box::new(input::Input::new(rx, worker_term)),
            lim.max as u64,
            Box::new(output::Output(out_tx, deadline)),
        )
    });
    tokio::spawn(flow::supervise(pump, worker, terminal));
    respond::response(out_rx, t, verdict).await
}
