use super::node::Node;
impl Node {
    pub fn change(&mut self, key: &str, old: Option<u64>, new: Option<u64>) {
        if let Some(total) = &self.prefix {
            total.lock().unwrap().update(old, new);
        }
        let mut node = self;
        for byte in key.bytes() {
            let Some(child) = node.children.get_mut(&byte) else {
                return;
            };
            node = child;
            if let Some(total) = &node.prefix {
                total.lock().unwrap().update(old, new);
            }
        }
        if let Some(total) = &node.exact {
            total.lock().unwrap().update(old, new);
        }
    }
}
