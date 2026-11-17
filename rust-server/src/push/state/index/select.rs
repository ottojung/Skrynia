use super::SubIndex;

impl SubIndex {
    pub fn matching(&self, key: &str, event: &str) -> Vec<String> {
        self.by_key
            .get(key)
            .and_then(|events| events.get(event))
            .map(|ids| ids.iter().cloned().collect())
            .unwrap_or_default()
    }
}
