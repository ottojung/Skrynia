mod execute;
use super::replies;
use crate::{Result, State, bad};
use axum::body::to_bytes;
use axum::extract::Request;
use axum::http::{Method, StatusCode};
use axum::response::Response;
use std::sync::Arc;

pub async fn route(state: Arc<State>, path: &str, req: Request) -> Result<Response> {
    // Issuer-only routes return an invalid-issuer error even without a token.
    let issuer_route = (req.method() == Method::POST && path.ends_with("/users"))
        || (req.method() == Method::DELETE && path.starts_with("/grants/"));
    let token = match replies::token(req.headers()) {
        Ok(value) => value.to_string(),
        Err(_) if issuer_route => String::new(),
        Err(error) => return Err(error),
    };
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
