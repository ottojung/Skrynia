use super::{SubIndex, node::Node};
impl SubIndex {
    pub fn matching(&self, key: &str, event: &str) -> Vec<String> {
        let mut result = Vec::new();
        let mut node = &self.root;
        node.include_prefix(event, &mut result);
        for byte in key.bytes() {
            let Some(next) = node.children.get(&byte) else {
                return result;
            };
            node = next;
            node.include_prefix(event, &mut result);
        }
        if let Some(ids) = node.exact.get(event) {
            result.extend(ids.iter().cloned());
        }
        result
    }
}
impl Node {
    pub fn include_prefix(&self, event: &str, result: &mut Vec<String>) {
        if let Some(ids) = self.prefix.get(event) {
            result.extend(ids.iter().cloned());
        }
    }
}
