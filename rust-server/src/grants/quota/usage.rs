use crate::Snapshot;
impl Snapshot {
    pub fn usage_authorized(&self, id: &str, old: Option<u64>, size: u64) -> (u64, u64) {
        let mut total = *self.quota.by_grant[id].lock().unwrap();
        total.update(old, Some(size));
        total.add(self.quota.metadata.get(id).copied().unwrap_or_default());
        (total.bytes, total.objects)
    }
    pub fn usage(&self, id: &str, update: Option<(&str, Option<u64>, u64)>) -> (u64, u64) {
        let grant = &self.grants[id];
        let mut total = *self.quota.by_grant[id].lock().unwrap();
        if let Some((key, old, size)) = update {
            if crate::matches(&grant.locations, key) {
                total.update(old, Some(size));
            }
        }
        total.add(self.quota.metadata.get(id).copied().unwrap_or_default());
        (total.bytes, total.objects)
    }
}
