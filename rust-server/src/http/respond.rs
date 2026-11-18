use axum::{
    http::{StatusCode, header::CONTENT_TYPE},
    response::{IntoResponse, Response},
};
pub fn vm(result: Result<Result<Vec<u8>, i32>, tokio::task::JoinError>) -> Response {
    match result {
        Ok(Ok(out)) => (
            StatusCode::OK,
            [(CONTENT_TYPE, "application/octet-stream")],
            out,
        )
            .into_response(),
        _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
pub fn upload(result: Result<std::io::Result<()>, tokio::task::JoinError>) -> Option<Response> {
    match result {
        Ok(Ok(())) => None,
        Ok(Err(e)) if e.kind() == std::io::ErrorKind::InvalidData => {
            Some(StatusCode::PAYLOAD_TOO_LARGE.into_response())
        }
        _ => Some(StatusCode::BAD_REQUEST.into_response()),
    }
}
