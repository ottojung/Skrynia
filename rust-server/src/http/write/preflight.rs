use crate::{Kind, Result, State};
pub fn exists(state: &State, key: &str) -> Result<()> {
    let data = state.data.lock().map_err(|_| crate::internal())?;
    state.require_healthy()?;
    if !data.objects.contains_key(key) {
        return Err(crate::missing("not_found"));
    }
    Ok(())
}

pub fn authorize(state: &State, key: &str, kind: Kind, token: &str) -> Result<()> {
    let data = state.data.lock().map_err(|_| crate::internal())?;
    state.require_healthy()?;
    data.grant_for(token, kind.operation(), key)?;
    Ok(())
}

pub async fn exists_async(state: std::sync::Arc<State>, key: String) -> Result<()> {
    tokio::task::spawn_blocking(move || exists(&state, &key))
        .await
        .map_err(|_| crate::internal())?
}
pub async fn authorize_async(
    state: std::sync::Arc<State>,
    key: String,
    kind: Kind,
    token: String,
) -> Result<()> {
    tokio::task::spawn_blocking(move || authorize(&state, &key, kind, &token))
        .await
        .map_err(|_| crate::internal())?
}
