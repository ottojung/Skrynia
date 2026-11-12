mod error;
mod grants;
mod http;
mod net;
mod push;
mod store;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let server = store::State::open()?;
    let address = std::env::var("SKRYNIA_HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let port = std::env::var("SKRYNIA_PORT").unwrap_or_else(|_| "17380".into());
    let listener = tokio::net::TcpListener::bind(format!("{address}:{port}")).await?;
    let router = http::router(server);
    let options = net::NetworkOptions::load()?;
    let listener = net::BoundedListener::new(listener, options);
    axum::serve(listener, router)
        .with_graceful_shutdown(net::shutdown_signal())
        .await?;
    Ok(())
}

// Public crate-level vocabulary keeps nested modules independent of topology.
pub use error::{Fail, Result, bad, forbidden, internal, missing};
pub use grants::{Grant, matches, random_id, sha, sha_file, subset, valid_key, valid_scope};
pub use grants::{OPERATIONS, recover_pending};
pub use store::disk::{sync_dir, write_json};
pub use store::{Snapshot, State};

pub use grants::{valid_identifier, valid_sha256};
pub use push::{
    Notification, PushConfig, PushKeys, PushState, Subscription, Vapid, endpoint_allowed,
    load_push, load_vapid, parse_endpoint as parse_push_endpoint, public_address, validate_push,
};

pub use grants::{random_secret, secure_match};
pub use push::{
    PushInput, parse_push_input, persist_push_subscription, route as push_route, same_push_events,
    valid_push_keys,
};

pub use push::{push_delay, push_enqueue, push_now, push_preflight};

pub use push::{PushFailure, push_send};

pub use push::{push_finish, push_run};

pub use http::ListParams;

pub use push::prune_orphan_notifications;
