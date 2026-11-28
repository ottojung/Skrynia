use super::{Common, collect, flow, input, output, respond};
use axum::{
    extract::Request,
    response::{IntoResponse, Response},
};
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
        out_tx,
        out_rx,
        deadline,
        terminal,
    } = common;
    let body = request.into_body();
    let bytes = match collect::buffered(body, lim.max, lim.inactivity, lim.total, &t).await {
        Ok(b) => b,
        Err(r) => return r.recorded(&t).into_response(),
    };
    let (tx, rx) = mpsc::channel(1);
    if !bytes.is_empty() {
        let _ = tx.send(Ok(bytes)).await;
    }
    drop(tx);
    let worker_term = t.clone();
    let worker = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        kernel.invoke_stream(
            Box::new(input::Input::new(rx, worker_term)),
            lim.max as u64,
            Box::new(output::Output(out_tx, deadline)),
        )
    });
    tokio::spawn(flow::supervise_done(worker, terminal));
    respond::response(out_rx, t).await
}

#[cfg(test)]
#[path = "preflight_fixture.rs"]
mod preflight_fixture;
#[cfg(test)]
#[path = "preflight_tests.rs"]
mod tests;
