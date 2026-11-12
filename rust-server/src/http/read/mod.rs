mod file;
mod headers;
mod metadata;
mod response;
use super::{auth, range, util};
use crate::{Result, State};
use axum::extract::Request;
use axum::response::Response;
use std::sync::Arc;

pub async fn read(state: Arc<State>, raw: &str, req: Request) -> Result<Response> {
    let key = auth::key(raw)?;
    let (file, size, sha) = file::open(state, &key).await?;
    let tag = format!("\"{sha}\"");
    let selection = headers::select(req.headers(), &tag, size);
    response::respond(file, &key, &tag, size, selection, req.method())
}
