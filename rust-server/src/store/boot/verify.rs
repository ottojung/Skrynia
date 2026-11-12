use crate::State;
use std::io;
pub fn finish(state: &State) -> io::Result<()> {
    state
        .data
        .lock()
        .map_err(|_| io::Error::other("poisoned"))?
        .validate()?;
    state.ensure_root()?;
    crate::journal_recover(&state.dir)?;
    Ok(())
}
