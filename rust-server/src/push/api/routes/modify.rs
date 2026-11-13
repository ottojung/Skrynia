use crate::{Fail, Result, State};
use axum::body::to_bytes;
use axum::extract::Request;
use axum::http::{Method, StatusCode};
use axum::response::{IntoResponse, Response};
use std::sync::Arc;
pub async fn run(
    state: Arc<State>,
    req: Request,
    id: String,
    cap: Option<String>,
) -> Result<Response> {
    let cap = cap.unwrap_or_default();
    if req.method() == Method::DELETE {
        tokio::task::spawn_blocking(move || state.remove_subscription(&id, &cap))
            .await
            .map_err(|_| crate::internal())??;
        return Ok(axum::Json(serde_json::json!({"ok":true})).into_response());
    }
    if req.method() == Method::PUT {
        let body = to_bytes(req.into_body(), 8192)
            .await
            .map_err(|_| crate::bad("request_too_large"))?;
        let value: serde_json::Value =
            serde_json::from_slice(&body).map_err(|_| crate::bad("invalid_json"))?;
        if !value.is_object() {
            return Err(crate::bad("invalid_json_object"));
        }
        super::patch::validate(&value)?;
        tokio::task::spawn_blocking(move || state.update_subscription(&id, &cap, &value))
            .await
            .map_err(|_| crate::internal())??;
        return Ok(axum::Json(serde_json::json!({"ok":true})).into_response());
    }
    Err(Fail(StatusCode::METHOD_NOT_ALLOWED, "method_not_allowed"))
}
