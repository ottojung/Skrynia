use super::{list, read, replies, write};
use crate::{Result, State};
use axum::extract::Request;
use axum::http::{Method, StatusCode};
use axum::response::Response;
use std::sync::Arc;

pub async fn route(state: Arc<State>, path: &str, req: Request) -> Result<Response> {
    if path == "/store" && req.method() == Method::GET {
        return list::listing(state, req).await;
    }
    if let Some(name) = path.strip_prefix("/store/") {
        if req.method() == Method::GET || req.method() == Method::HEAD {
            return read::read(state, name, req).await;
        }
        if [Method::POST, Method::PUT, Method::DELETE].contains(req.method()) {
            return write::mutation(state, name.to_string(), req).await;
        }
    }
    Ok(replies::json(
        StatusCode::METHOD_NOT_ALLOWED,
        serde_json::json!({"error":"method_not_allowed"}),
    ))
}
