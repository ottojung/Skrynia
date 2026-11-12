mod page;
mod params;
use super::replies;
use crate::{Result, State};
use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::Response;
use std::sync::Arc;

pub async fn listing(state: Arc<State>, req: Request) -> Result<Response> {
    let token = replies::token(req.headers())?;
    let params = params::parse(req.uri().query().unwrap_or(""))?;
    let snapshot = state.data.lock().map_err(|_| crate::internal())?;
    let response = page::list(&snapshot, token, params)?;
    Ok(replies::json(StatusCode::OK, response))
}
