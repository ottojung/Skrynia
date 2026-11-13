use super::{Shared, Usage};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
#[derive(Default)]
pub struct Node {
    pub(super) children: HashMap<u8, Node>,
    pub(super) prefix: Option<Shared>,
    pub(super) exact: Option<Shared>,
}
impl Node {
    fn descend_mut(&mut self, bytes: &[u8]) -> &mut Self {
        let mut node = self;
        for byte in bytes {
            node = node.children.entry(*byte).or_default();
        }
        node
    }
    pub(super) fn descend(&self, bytes: &[u8]) -> Option<&Self> {
        let mut node = self;
        for byte in bytes {
            node = node.children.get(byte)?;
        }
        Some(node)
    }
    pub fn insert(&mut self, scope: &str, total: Usage) -> Shared {
        if let Some(value) = self.shared(scope) {
            return value;
        }
        let (name, prefix) = scope
            .strip_suffix('*')
            .map_or((scope, false), |s| (s, true));
        let value = Arc::new(Mutex::new(total));
        let node = self.descend_mut(name.as_bytes());
        if prefix {
            node.prefix = Some(value.clone());
        } else {
            node.exact = Some(value.clone());
        }
        value
    }
}
