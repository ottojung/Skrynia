use crate::{QuotaIndex, State};
use std::io;
pub fn finish(state: &State) -> io::Result<()> {
    state
        .data
        .lock()
        .map_err(|_| io::Error::other("poisoned"))?
        .validate()?;
    state.ensure_root()?;
    {
        let data = state
            .data
            .lock()
            .map_err(|_| io::Error::other("poisoned"))?;
        crate::validate_push(&data.push, &data.grants, &state.push_config)?;
    }
    state.recover_journal()?;
    let mut data = state
        .data
        .lock()
        .map_err(|_| io::Error::other("poisoned"))?;
    data.quota = QuotaIndex::from_disk(&state.dir, &data.grants, &data.push)?;
    Ok(())
}
