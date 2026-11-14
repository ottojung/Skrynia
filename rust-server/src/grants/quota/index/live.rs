use super::{QuotaIndex, Usage};
use crate::{Grant, PushState};
use std::collections::HashMap;
impl QuotaIndex {
    pub fn rebuild_live(&self, grants: &HashMap<String, Grant>, push: &PushState) -> Self {
        let mut rebuilt = Self::default();
        for grant in grants.values() {
            if !rebuilt.scopes.has(&grant.locations) {
                let total = self.scopes.get(&grant.locations);
                rebuilt.scopes.insert(&grant.locations, total);
            }
            rebuilt.attach_grant(&grant.id, &grant.locations);
        }
        for grant in grants.values() {
            let bytes = 2048 + grant.users.len().saturating_sub(1) as u64 * 128;
            rebuilt.change_metadata(grants, &grant.id, Usage { bytes, objects: 0 }, true);
        }
        for sub in push.subscriptions.values() {
            rebuilt.change_metadata(
                grants,
                &sub.grant_id,
                Usage {
                    bytes: 4096,
                    objects: 1,
                },
                true,
            );
        }
        rebuilt
    }
}
