use crate::{Kind, Result, State};
pub fn exists(state: &State, key: &str) -> Result<()> {
    let _data = state.data.lock().map_err(|_| crate::internal())?;
    state.require_healthy()?;
    let path = state.dir.join("store").join(key);
    crate::open_regular(&path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            crate::missing("not_found")
        } else {
            crate::internal()
        }
    })?;
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
