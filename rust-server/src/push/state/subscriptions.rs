use super::PushState;
use crate::Subscription;
impl PushState {
    pub fn reindex(&mut self) {
        self.index = super::SubIndex::default();
        for sub in self.subscriptions.values() {
            self.index.insert(sub);
        }
    }
    pub fn put_sub(&mut self, sub: Subscription) {
        if let Some(old) = self.subscriptions.insert(sub.id.clone(), sub.clone()) {
            self.index.remove(&old);
        }
        self.index.insert(&sub);
    }
    pub fn remove_sub(&mut self, id: &str) -> Option<Subscription> {
        let old = self.subscriptions.remove(id)?;
        self.index.remove(&old);
        Some(old)
    }
}
