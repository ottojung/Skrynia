mod build;
mod common;
pub use build::validate as validate_build_info;
mod storage;
use super::{grants, list, read, replies, write};
use crate::State;
use axum::extract::{Request, State as AxumState};
use axum::http::{Method, StatusCode};
use axum::response::{IntoResponse, Response};
use std::sync::Arc;

pub async fn dispatch(AxumState(state): AxumState<Arc<State>>, req: Request) -> Response {
    if let Some(mut response) = common::failed(&state) {
        replies::cors(&mut response);
        return response;
    }
    let mut response = route(state, req)
        .await
        .unwrap_or_else(IntoResponse::into_response);
    replies::cors(&mut response);
    response
}
async fn route(state: Arc<State>, req: Request) -> crate::Result<Response> {
    if req.method() == Method::OPTIONS {
        return Ok(replies::preflight());
    }
    let path = common::mounted_path(&state.base, req.uri().path());
    if let Some(response) = common::public(&path, req.method()) {
        return Ok(response);
    }
    if path == "/store" || path.starts_with("/store/") {
        return storage::route(state, &path, req).await;
    }
    if path.starts_with("/grants") {
        return grants::route(state, &path, req).await;
    }
    if path.starts_with("/push") {
        return crate::push_route(state, &path, req).await;
    }
    Ok(replies::json(
        StatusCode::NOT_FOUND,
        serde_json::json!({"error":"not_found"}),
    ))
}
