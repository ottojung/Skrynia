use super::{auth, replies};
use crate::State;
use crate::{Fail, Result};
use axum::body::to_bytes;
use axum::extract::Request;
use axum::http::{Method, StatusCode};
use axum::response::Response;
use std::sync::Arc;

pub async fn mutation(state: Arc<State>, raw_name: String, req: Request) -> Result<Response> {
    let key = auth::key(&raw_name)?;
    let expected = auth::precondition(req.headers(), req.method())?;
    let token = replies::token(req.headers())?.to_string();
    let method = req.method().clone();
    let kind = match method {
        Method::POST => "create",
        Method::PUT => "replace",
        _ => "delete",
    };
    let operation = if kind == "replace" { "modify" } else { kind };
    {
        let data = state.data.lock().map_err(|_| crate::internal())?;
        data.grant_for(&token, operation, &key)?;
    }
    let body = if method == Method::DELETE {
        Vec::new()
    } else {
        to_bytes(req.into_body(), state.max_object)
            .await
            .map_err(|_| Fail(StatusCode::PAYLOAD_TOO_LARGE, "request_too_large"))?
            .to_vec()
    };
    let outcome = tokio::task::spawn_blocking(move || {
        state.change(&key, kind, &body, expected.as_deref(), &token)
    })
    .await
    .map_err(|_| crate::internal())?;
    let pending = outcome?;
    Ok(replies::json(
        if method == Method::POST {
            StatusCode::CREATED
        } else {
            StatusCode::OK
        },
        if pending {
            serde_json::json!({"ok":true,"notificationPending":true})
        } else {
            serde_json::json!({"ok":true})
        },
    ))
}
