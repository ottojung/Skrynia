mod preflight;
mod upload;
use super::{auth, replies};
use crate::{Kind, Result, State};
use axum::extract::Request;
use axum::http::{Method, StatusCode};
use axum::response::Response;
use std::sync::Arc;

pub async fn mutation(state: Arc<State>, raw_name: String, req: Request) -> Result<Response> {
    let key = auth::key(&raw_name, state.max_key_length)?;
    let expected = auth::precondition(req.headers(), req.method())?;
    // POST and PUT instead reject unauthorized upload bodies before buffering.
    if req.method() == Method::DELETE {
        preflight::exists_async(Arc::clone(&state), key.clone()).await?;
    }
    let token = replies::token(req.headers())?.to_string();
    let method = req.method().clone();
    let kind = match method {
        Method::POST => Kind::Create,
        Method::PUT => Kind::Replace,
        _ => Kind::Delete,
    };
    preflight::authorize_async(Arc::clone(&state), key.clone(), kind, token.clone()).await?;
    let upload = if method == Method::DELETE {
        upload::Upload::empty(Arc::clone(&state.upload))
    } else {
        upload::read(&state, req).await?
    };
    let outcome = tokio::task::spawn_blocking(move || {
        let committed = state.change(&key, kind, &upload.bytes, expected.as_deref(), &token);
        drop(upload);
        committed
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
