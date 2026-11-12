mod error;
mod grants;
mod http;
mod push;
mod store;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let server = store::State::open()?;
    let address = std::env::var("SKRYNIA_HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let port = std::env::var("SKRYNIA_PORT").unwrap_or_else(|_| "17380".into());
    let listener = tokio::net::TcpListener::bind(format!("{address}:{port}")).await?;
    let router = http::router(server);
    axum::serve(listener, router).await?;
    Ok(())
}

// Public crate-level vocabulary keeps nested modules independent of topology.
pub use error::{Fail, Result, bad, forbidden, internal, missing};
pub use grants::{Grant, matches, random_id, sha, sha_file, subset, valid_key, valid_scope};
pub use grants::{OPERATIONS, recover_pending};
pub use store::disk::{sync_dir, write_json};
pub use store::{Snapshot, State};
