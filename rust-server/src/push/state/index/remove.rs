use super::node::Node;
impl Node {
    pub fn remove(&mut self, scope: &str, event: &str, id: &str) {
        let (name, prefix) = scope
            .strip_suffix('*')
            .map_or((scope, false), |s| (s, true));
        self.remove_walk(name.as_bytes(), prefix, event, id);
    }
    fn remove_walk(&mut self, bytes: &[u8], prefix: bool, event: &str, id: &str) {
        if let Some((&first, rest)) = bytes.split_first() {
            if let Some(child) = self.children.get_mut(&first) {
                child.remove_walk(rest, prefix, event, id);
                if child.empty() {
                    self.children.remove(&first);
                }
            }
        } else {
            let events = if prefix {
                &mut self.prefix
            } else {
                &mut self.exact
            };
            if let Some(ids) = events.get_mut(event) {
                ids.remove(id);
                if ids.is_empty() {
                    events.remove(event);
                }
            }
        }
    }
    fn empty(&self) -> bool {
        self.children.is_empty() && self.prefix.is_empty() && self.exact.is_empty()
    }
}
