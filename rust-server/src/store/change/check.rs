use super::Kind;
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
    kind: Kind,
    body: &[u8],
    path: &Path,
    token: &str,
    mode: Mode<'_>,
) -> Result<()> {
    let old = data.objects.get(key).copied();
    if kind == Kind::Create && old.is_some() {
        return Err(Fail(StatusCode::CONFLICT, "already_exists"));
    }
    if kind != Kind::Create && old.is_none() {
        return Err(crate::missing("not_found"));
    }
    let grant = data.grant_for(token, kind.operation(), key)?;
    if let Mode::Final(Some(expected)) = mode {
        let observed = crate::sha_file(path).map_err(|_| crate::internal())?;
        if observed != expected {
            return Err(Fail(StatusCode::PRECONDITION_FAILED, "content_mismatch"));
        }
    }
    if kind != Kind::Delete && matches!(mode, Mode::Final(_)) {
        data.check_quota(&grant.id, key, body.len() as u64)?;
    }
    Ok(())
}
