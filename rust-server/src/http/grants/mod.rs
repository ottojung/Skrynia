mod action;
mod execute;
use super::replies;
use crate::{Result, State, bad, missing};
use axum::body::to_bytes;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::Response;
use std::sync::Arc;
pub async fn route(state: Arc<State>, path: &str, req: Request) -> Result<Response> {
    let action = action::Action::parse(path, req.method()).ok_or_else(|| missing("not_found"))?;
    let token = match replies::token(req.headers()) {
        Ok(value) => value.to_string(),
        Err(_) if action.issuer_only() => String::new(),
        Err(error) => return Err(error),
    };
    let spec = if matches!(action, action::Action::Create) {
        let bytes = to_bytes(req.into_body(), 8192)
            .await
            .map_err(|_| bad("request_too_large"))?;
        Some(serde_json::from_slice(&bytes).map_err(|_| bad("invalid_json"))?)
    } else {
        None
    };
    let result = tokio::task::spawn_blocking(move || execute::run(&state, action, &token, spec))
        .await
        .map_err(|_| crate::internal())??;
    Ok(replies::json(StatusCode::OK, result))
}
