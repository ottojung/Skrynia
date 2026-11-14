use std::collections::{HashMap, HashSet};
#[derive(Default)]
pub struct Node {
    pub(super) children: HashMap<u8, Node>,
    pub(super) prefix: HashMap<String, HashSet<String>>,
    pub(super) exact: HashMap<String, HashSet<String>>,
}
impl Node {
    pub fn insert(&mut self, scope: &str, event: &str, id: &str) {
        let (name, prefix) = scope
            .strip_suffix('*')
            .map_or((scope, false), |s| (s, true));
        let mut node = self;
        for byte in name.bytes() {
            node = node.children.entry(byte).or_default();
        }
        let events = if prefix {
            &mut node.prefix
        } else {
            &mut node.exact
        };
        events
            .entry(event.to_string())
            .or_default()
            .insert(id.into());
    }
}
