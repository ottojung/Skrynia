use super::{
    input, pump,
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
    let sender = tokio::spawn(pump::run(request.into_body(), tx, max));
    let kernel = app.kernel;
    let call = tokio::task::spawn_blocking(move || {
        kernel.invoke_stream(Box::new(input::Input::new(rx)), max as u64)
    })
    .await;
    let uploaded = sender.await;
    if matches!(uploaded, Ok(Err(ref e)) if e.kind() == std::io::ErrorKind::InvalidData) {
        return StatusCode::PAYLOAD_TOO_LARGE.into_response();
    }
    if !matches!(uploaded, Ok(Ok(()))) {
        return StatusCode::BAD_REQUEST.into_response();
    }
    match call {
        Ok(Ok(out)) => (
            StatusCode::OK,
            [(axum::http::header::CONTENT_TYPE, "application/octet-stream")],
            out,
        )
            .into_response(),
        _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
