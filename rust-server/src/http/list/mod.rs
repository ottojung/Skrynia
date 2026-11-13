mod cursor;
mod page;
mod params;
use super::replies;
use crate::{Result, State};
use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::Response;
pub use params::Params as ListParams;
use std::sync::Arc;

pub async fn listing(state: Arc<State>, req: Request) -> Result<Response> {
    let token = replies::token(req.headers())?.to_string();
    let params = params::parse(req.uri().query().unwrap_or(""))?;
    let response = tokio::task::spawn_blocking(move || {
        let snapshot = state.data.lock().map_err(|_| crate::internal())?;
        state.require_healthy()?;
        page::list(&snapshot, &token, params)
    })
    .await
    .map_err(|_| crate::internal())??;
    Ok(replies::json(StatusCode::OK, response))
}
