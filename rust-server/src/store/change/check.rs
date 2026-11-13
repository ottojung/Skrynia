use crate::{Fail, Result, Snapshot};
use axum::http::StatusCode;
use std::path::Path;

pub enum Mode<'a> {
    Initial,
    Final(Option<&'a str>),
}
pub fn preflight(
    data: &Snapshot,
    key: &str,
    kind: &str,
    body: &[u8],
    path: &Path,
    token: &str,
    mode: Mode<'_>,
) -> Result<()> {
    let old = data.objects.get(key).copied();
    if kind == "create" && old.is_some() {
        return Err(Fail(StatusCode::CONFLICT, "already_exists"));
    }
    if kind != "create" && old.is_none() {
        return Err(crate::missing("not_found"));
    }
    let operation = if kind == "replace" { "modify" } else { kind };
    let grant = data.grant_for(token, operation, key)?;
    if let Mode::Final(Some(expected)) = mode {
        let observed = crate::sha_file(path).map_err(|_| crate::internal())?;
        if observed != expected {
            return Err(Fail(StatusCode::PRECONDITION_FAILED, "content_mismatch"));
        }
    }
    if kind != "delete" && matches!(mode, Mode::Final(_)) {
        data.check_quota(&grant.id, key, body.len() as u64)?;
    }
    Ok(())
}
