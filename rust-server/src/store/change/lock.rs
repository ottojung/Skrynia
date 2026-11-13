use super::check;
use crate::{Result, Snapshot, State};
use std::sync::MutexGuard;
pub fn checked<'a>(
    state: &'a State,
    key: &str,
    kind: &str,
    body: &[u8],
    expected: Option<&str>,
    token: &str,
    strict: bool,
) -> Result<MutexGuard<'a, Snapshot>> {
    let data = state.data.lock().map_err(|_| crate::internal())?;
    state.require_healthy()?;
    let path = state.dir.join("store").join(key);
    let mode = if strict {
        check::Mode::Final(expected)
    } else {
        check::Mode::Initial
    };
    check::preflight(&data, key, kind, body, &path, token, mode)?;
    crate::push_preflight(&data, key, kind, state.push_config.max_outbox)?;
    Ok(data)
}
