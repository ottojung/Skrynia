mod auth;
mod entry;
mod grants;
mod list;
mod range;
mod read;
mod replies;
mod util;
mod write;

pub fn router(state: crate::store::State) -> axum::Router {
    let state = std::sync::Arc::new(state);
    if !state.push_config.manual {
        tokio::spawn(crate::push_run(std::sync::Arc::clone(&state)));
    }
    axum::Router::new()
        .fallback(entry::dispatch)
        .with_state(state)
}
