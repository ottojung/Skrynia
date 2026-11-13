use super::{Kind, check};
use crate::{Result, Snapshot, State};
use std::sync::MutexGuard;
pub fn checked<'a>(
    state: &'a State,
    key: &str,
    kind: Kind,
    body: &[u8],
    token: &str,
    mode: check::Mode<'_>,
) -> Result<MutexGuard<'a, Snapshot>> {
    let data = state.data.lock().map_err(|_| crate::internal())?;
    state.require_healthy()?;
    let path = state.dir.join("store").join(key);
    check::preflight(&data, key, kind, body, &path, token, mode)?;
    crate::push_preflight(&data, key, kind.event(), state.push_config.max_outbox)?;
    Ok(data)
}
