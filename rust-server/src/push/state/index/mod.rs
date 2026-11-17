mod select;
#[cfg(test)]
mod tests;

use crate::Subscription;
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub struct SubIndex {
    by_key: HashMap<String, HashMap<String, HashSet<String>>>,
}

impl SubIndex {
    pub fn insert(&mut self, sub: &Subscription) {
        let events = self.by_key.entry(sub.locations.clone()).or_default();
        for event in &sub.events {
            events
                .entry(event.clone())
                .or_default()
                .insert(sub.id.clone());
        }
    }

    pub fn remove(&mut self, sub: &Subscription) {
        let Some(events) = self.by_key.get_mut(&sub.locations) else {
            return;
        };
        for event in &sub.events {
            let Some(ids) = events.get_mut(event) else {
                continue;
            };
            ids.remove(&sub.id);
            if ids.is_empty() {
                events.remove(event);
            }
        }
        if events.is_empty() {
            self.by_key.remove(&sub.locations);
        }
    }
}
