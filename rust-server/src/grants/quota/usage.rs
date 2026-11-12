use super::objects;
use crate::Snapshot;
const GRANT_COST: u64 = 2048;
const EXTRA_USER_COST: u64 = 128;

impl Snapshot {
    pub fn usage(&self, id: &str, update: Option<(&str, u64)>) -> (u64, u64) {
        let grant = &self.grants[id];
        let (mut bytes, objects) = objects::sum(&self.objects, &grant.locations, update);
        let descendants = self.children_of(id);
        let ids = descendants
            .iter()
            .map(|g| g.id.as_str())
            .collect::<std::collections::HashSet<_>>();
        for descendant in descendants {
            bytes += GRANT_COST + descendant.users.len().saturating_sub(1) as u64 * EXTRA_USER_COST;
        }
        let mut objects = objects;
        for sub in self.push.subscriptions.values() {
            if ids.contains(sub.grant_id.as_str()) {
                bytes += 4096;
                objects += 1;
            }
        }
        (bytes, objects)
    }
}
