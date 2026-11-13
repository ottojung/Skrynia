mod create;
mod modify;
use crate::{Fail, Result, State};
use axum::extract::Request;
use axum::http::{Method, StatusCode};
use axum::response::{IntoResponse, Response};
use std::sync::Arc;
pub async fn route(state: Arc<State>, path: &str, req: Request) -> Result<Response> {
    if path == "/push/vapid" && req.method() == Method::GET {
        return Ok(
            axum::Json(serde_json::json!({"publicKey":state.vapid.public_key})).into_response(),
        );
    }
    let method = req.method().clone();
    let cap = req
        .headers()
        .get("x-skrynia-capability")
        .and_then(|h| h.to_str().ok())
        .map(str::to_string);
    let token = req
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(str::to_string);
    if path == "/push/subscriptions" && method == Method::POST {
        return create::run(state, req, cap, token).await;
    }
    if let Some(id) = path.strip_prefix("/push/subscriptions/")
        && crate::valid_identifier(id)
    {
        return modify::run(state, req, id.into(), cap).await;
    }
    Err(Fail(StatusCode::NOT_FOUND, "not_found"))
}
