use super::{QuotaIndex, Usage};
use crate::{Grant, PushState};
use std::{collections::HashMap, io, path::Path};
impl QuotaIndex {
    pub fn from_disk(
        dir: &Path,
        grants: &HashMap<String, Grant>,
        push: &PushState,
    ) -> io::Result<Self> {
        let mut result = Self::default();
        for grant in grants.values() {
            if !result.scopes.has(&grant.locations) {
                result.scopes.insert(&grant.locations, Usage::default());
            }
        }
        for grant in grants.values() {
            result.attach_grant(&grant.id, &grant.locations);
        }
        result.scan_objects(dir)?;
        for grant in grants.values() {
            let extra = grant.users.len().saturating_sub(1) as u64 * 128 + 2048;
            result.change_metadata(
                grants,
                &grant.id,
                Usage {
                    bytes: extra,
                    objects: 0,
                },
                true,
            );
        }
        for sub in push.subscriptions.values() {
            result.change_metadata(
                grants,
                &sub.grant_id,
                Usage {
                    bytes: 4096,
                    objects: 1,
                },
                true,
            );
        }
        Ok(result)
    }
}
