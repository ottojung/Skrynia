use super::objects;
use crate::Snapshot;
const GRANT_COST: u64 = 2048;
const EXTRA_USER_COST: u64 = 128;

impl Snapshot {
    pub fn usage(&self, id: &str, update: Option<(&str, u64)>) -> (u64, u64) {
        let grant = &self.grants[id];
        let (mut bytes, objects) = objects::sum(&self.objects, &grant.locations, update);
        for descendant in self.children_of(id) {
            bytes += GRANT_COST + descendant.users.len().saturating_sub(1) as u64 * EXTRA_USER_COST;
        }
        (bytes, objects)
    }
}
