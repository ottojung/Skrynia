use super::{Kind, check, lock};
use crate::{Result, State};
pub fn staged(
    state: &State,
    key: &str,
    kind: Kind,
    body: &[u8],
    token: &str,
) -> Result<Option<super::super::staging::Staged>> {
    drop(lock::checked(
        state,
        key,
        kind,
        body,
        token,
        check::Mode::Initial,
    )?);
    if kind == Kind::Delete {
        return Ok(None);
    }
    super::super::staging::Staged::prepare(&state.dir, body)
        .map(Some)
        .map_err(|error| crate::error::transient_io(&error))
}
