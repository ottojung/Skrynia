mod index;
mod keys;
mod load;
mod outbox;
mod persist;
mod subscriptions;
mod validate;
use crate::{Pending, Subscription};
pub use index::SubIndex;
use std::collections::HashMap;
use std::{io, path::Path};
#[derive(Default)]
pub struct PushState {
    pub subscriptions: HashMap<String, Subscription>,
    pub outbox: HashMap<String, Pending>,
    pub index: SubIndex,
}
pub fn load_push(root: &Path) -> io::Result<PushState> {
    let subscriptions = load::read(&root.join("state/push-subs"))?;
    let outbox = outbox::read(root)?;
    if outbox.keys().any(|id| !subscriptions.contains_key(id)) {
        return Err(io::Error::other("orphan push notification"));
    }
    let mut state = PushState {
        subscriptions,
        outbox,
        index: SubIndex::default(),
    };
    state.reindex();
    Ok(state)
}
pub use outbox::read as read_outbox;
#[cfg(test)]
pub use persist::count_writes as outbox_write_count;
pub use persist::save as persist_outbox;
pub use validate::valid_keys;
pub use validate::validate_push;
