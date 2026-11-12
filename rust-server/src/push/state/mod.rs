mod load;
mod prune;
mod validate;
use crate::{Notification, Subscription};
use std::collections::HashMap;
use std::io;
use std::path::Path;

#[derive(Default)]
pub struct PushState {
    pub subscriptions: HashMap<String, Subscription>,
    pub outbox: HashMap<String, Notification>,
}
pub fn load_push(root: &Path) -> io::Result<PushState> {
    let subscriptions = load::read(&root.join("state/push-subs"))?;
    let outbox = load::read(&root.join("state/push/outbox"))?;
    Ok(PushState {
        subscriptions,
        outbox,
    })
}
pub use validate::validate_push;

pub use validate::valid_keys;

pub use prune::orphans as prune_orphan_notifications;
