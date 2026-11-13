mod load;
mod outbox;
mod persist;
mod validate;
use crate::{Pending, Subscription};
use std::collections::HashMap;
use std::io;
use std::path::Path;

#[derive(Default)]
pub struct PushState {
    pub subscriptions: HashMap<String, Subscription>,
    pub outbox: HashMap<String, Pending>,
}
pub fn load_push(root: &Path) -> io::Result<PushState> {
    let subscriptions = load::read(&root.join("state/push-subs"))?;
    let outbox = outbox::read(root)?;
    if outbox.keys().any(|id| !subscriptions.contains_key(id)) {
        return Err(io::Error::other("orphan push notification"));
    }
    Ok(PushState {
        subscriptions,
        outbox,
    })
}
pub use validate::validate_push;

pub use validate::valid_keys;

pub use outbox::read as read_outbox;
pub use persist::save as persist_outbox;

#[cfg(test)]
pub use persist::count_writes as outbox_write_count;
