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
    axum::Router::new()
        .fallback(entry::dispatch)
        .with_state(std::sync::Arc::new(state))
}
