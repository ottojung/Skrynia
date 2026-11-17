use super::{Kind, check};
use crate::{Result, Snapshot, State};
use std::sync::MutexGuard;
pub fn checked<'a>(
    state: &'a State,
    key: &str,
    kind: Kind,
    size: u64,
    token: &str,
    mode: check::Mode<'_>,
) -> Result<(MutexGuard<'a, Snapshot>, Option<u64>)> {
    let data = state.data.lock().map_err(|_| crate::internal())?;
    state.require_healthy()?;
    let path = state.dir.join("store").join(key);
    match super::super::paths::safe_parent(&state.dir, key) {
        Ok(_) => {}
        Err(e) if kind == Kind::Create && e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(crate::missing("not_found"));
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            return Err(crate::Fail(
                axum::http::StatusCode::CONFLICT,
                "path_conflict",
            ));
        }
        Err(_) => return Err(crate::internal()),
    }
    let old = check::preflight(&data, key, kind, size, &path, token, mode)?;
    crate::push_preflight(&data, key, kind.event(), state.push_config.max_outbox)?;
    Ok((data, old))
}
