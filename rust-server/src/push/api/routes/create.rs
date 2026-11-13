use crate::{Result, State};
use axum::body::to_bytes;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::sync::Arc;
pub async fn run(
    state: Arc<State>,
    req: Request,
    cap: Option<String>,
    token: Option<String>,
) -> Result<Response> {
    let body = to_bytes(req.into_body(), 8192)
        .await
        .map_err(|_| crate::bad("request_too_large"))?;
    let value: serde_json::Value =
        serde_json::from_slice(&body).map_err(|_| crate::bad("invalid_json"))?;
    if !value.is_object() {
        return Err(crate::bad("invalid_json_object"));
    }
    let input = crate::parse_push_input(&value, &state.push_config)?;
    let result = tokio::task::spawn_blocking(move || {
        state.subscribe(input, cap.as_deref(), token.as_deref())
    })
    .await
    .map_err(|_| crate::internal())??;
    Ok((
        StatusCode::CREATED,
        axum::Json(serde_json::json!({"ok":true,
        "id":result["id"],"capability":result["capability"],"deduped":result["deduped"]})),
    )
        .into_response())
}
