use super::{
    input, pump, respond,
    router::{App, max_req},
};
use axum::{
    extract::{Request, State},
    http::{Method, StatusCode},
    response::IntoResponse,
};
pub(super) async fn handle(State(app): State<App>, request: Request) -> impl IntoResponse {
    if request.method() != Method::POST || request.uri().path() != "/" {
        return StatusCode::NOT_FOUND.into_response();
    }
    let max = max_req();
    if request
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<usize>().ok())
        .is_some_and(|n| n > max)
    {
        return StatusCode::PAYLOAD_TOO_LARGE.into_response();
    }
    let (tx, rx) = tokio::sync::mpsc::channel(2);
    let mut sender = tokio::spawn(pump::run(request.into_body(), tx, max));
    let kernel = app.kernel;
    let mut call = tokio::task::spawn_blocking(move || {
        kernel.invoke_stream(Box::new(input::Input::new(rx)), max as u64)
    });
    let result = tokio::select! {
        biased;
        uploaded = &mut sender => {
            if let Some(response) = respond::upload(uploaded) {
                return response;
            }
            call.await
        }
        finished = &mut call => {
            sender.abort();
            finished
        }
    };
    respond::vm(result)
}
