mod execute;
use super::replies;
use crate::{Result, State, bad};
use axum::body::to_bytes;
use axum::extract::Request;
use axum::http::{Method, StatusCode};
use axum::response::Response;
use std::sync::Arc;

pub async fn route(state: Arc<State>, path: &str, req: Request) -> Result<Response> {
    let token = replies::token(req.headers())?.to_string();
    let method = req.method().clone();
    let spec = if method == Method::POST && path == "/grants" {
        let bytes = to_bytes(req.into_body(), 8192)
            .await
            .map_err(|_| bad("request_too_large"))?;
        let value: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|_| bad("invalid_json"))?;
        Some(value)
    } else {
        None
    };
    let path = path.to_string();
    let result =
        tokio::task::spawn_blocking(move || execute::run(&state, &path, method, &token, spec))
            .await
            .map_err(|_| crate::internal())??;
    Ok(replies::json(StatusCode::OK, result))
}
