use super::{Node, Shared, Usage};
impl Node {
    pub fn shared(&self, scope: &str) -> Option<Shared> {
        let (name, prefix) = scope
            .strip_suffix('*')
            .map_or((scope, false), |s| (s, true));
        self.descend(name.as_bytes())
            .and_then(|n| {
                if prefix {
                    n.prefix.as_ref()
                } else {
                    n.exact.as_ref()
                }
            })
            .cloned()
    }
    pub fn has(&self, scope: &str) -> bool {
        self.shared(scope).is_some()
    }
    pub fn get(&self, scope: &str) -> Usage {
        self.shared(scope)
            .map_or(Usage::default(), |value| *value.lock().unwrap())
    }
}
