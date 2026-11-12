use crate::{Snapshot, matches};
const OBJECT_COST: u64 = 512;
const GRANT_COST: u64 = 2048;
const EXTRA_USER_COST: u64 = 128;

impl Snapshot {
    pub fn usage(&self, id: &str, update: Option<(&str, u64)>) -> (u64, u64) {
        let grant = &self.grants[id];
        let mut bytes = 0u64;
        let mut objects = 0u64;
        for (key, size) in &self.objects {
            if matches(&grant.locations, key) {
                bytes += update
                    .filter(|(name, _)| name == key)
                    .map_or(*size, |(_, n)| n)
                    + OBJECT_COST;
                objects += 1;
            }
        }
        if let Some((key, size)) = update {
            if !self.objects.contains_key(key) && matches(&grant.locations, key) {
                bytes += size + OBJECT_COST;
                objects += 1;
            }
        }
        for descendant in self.children_of(id) {
            bytes += GRANT_COST + descendant.users.len().saturating_sub(1) as u64 * EXTRA_USER_COST;
        }
        (bytes, objects)
    }
}
