use crate::{Result, State};
pub fn exists(state: &State, key: &str) -> Result<()> {
    let data = state.data.lock().map_err(|_| crate::internal())?;
    state.require_healthy()?;
    if !data.objects.contains_key(key) {
        return Err(crate::missing("not_found"));
    }
    Ok(())
}

pub fn authorize(state: &State, key: &str, kind: &str, token: &str) -> Result<()> {
    let operation = if kind == "replace" { "modify" } else { kind };
    let data = state.data.lock().map_err(|_| crate::internal())?;
    state.require_healthy()?;
    data.grant_for(token, operation, key)?;
    Ok(())
}
