use super::replies;
use crate::State;
use crate::{Result, bad};
use axum::body::to_bytes;
use axum::extract::Request;
use axum::http::{Method, StatusCode};
use axum::response::Response;
use std::sync::Arc;

pub async fn route(state: Arc<State>, path: &str, req: Request) -> Result<Response> {
    let token = replies::token(req.headers())?.to_string();
    let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    let method = req.method().clone();
    let result = match (parts.as_slice(), method) {
        (["grants"], Method::POST) => {
            let bytes = to_bytes(req.into_body(), 8192)
                .await
                .map_err(|_| bad("request_too_large"))?;
            let spec: serde_json::Value =
                serde_json::from_slice(&bytes).map_err(|_| bad("invalid_json"))?;
            state.child(&token, &spec)?
        }
        (["grants", id], Method::GET) => state.inspect(id, &token)?,
        (["grants", id, "users"], Method::POST) => state.issue_user(id, &token)?,
        (["grants", id, "users", user], Method::DELETE) => {
            state.revoke_user(id, &token, user)?;
            serde_json::json!({"ok":true,"grantId":id,"userId":user})
        }
        (["grants", id], Method::DELETE) => {
            state.revoke_grant(id, &token)?;
            serde_json::json!({"ok":true,"grantId":id})
        }
        _ => return Err(crate::missing("not_found")),
    };
    Ok(replies::json(StatusCode::OK, result))
}
