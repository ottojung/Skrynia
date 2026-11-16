use super::{QuotaIndex, Usage};
use crate::{Grant, PushState};
use std::{
    collections::HashMap,
    io,
    path::Path,
    sync::{Arc, Mutex},
};
impl QuotaIndex {
    pub fn from_disk(
        dir: &Path,
        grants: &HashMap<String, Grant>,
        push: &PushState,
    ) -> io::Result<Self> {
        let mut result = Self::default();
        result.delta = Some(Arc::new(Mutex::new(super::super::DeltaIndex::new(dir)?)));
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
            let usage = Usage {
                bytes: 4096,
                objects: 1,
            };
            result.change_metadata(grants, &sub.grant_id, usage, true);
        }
        Ok(result)
    }
}
