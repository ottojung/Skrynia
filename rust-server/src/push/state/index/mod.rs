mod node;
mod remove;
mod select;
#[cfg(test)]
mod tests;
use crate::Subscription;
#[derive(Default)]
pub struct SubIndex {
    pub(super) root: node::Node,
}
impl SubIndex {
    pub fn insert(&mut self, sub: &Subscription) {
        if sub.locations.is_empty() {
            return;
        }
        for event in &sub.events {
            self.root.insert(&sub.locations, event, &sub.id);
        }
    }
    pub fn remove(&mut self, sub: &Subscription) {
        if sub.locations.is_empty() {
            return;
        }
        for event in &sub.events {
            self.root.remove(&sub.locations, event, &sub.id);
        }
    }
}
