use super::Value;
use std::{collections::HashMap, io};
#[derive(Default)]
pub struct Node {
    total: Value,
    children: HashMap<u8, Node>,
}
impl Node {
    pub fn change(&mut self, key: &[u8], value: Value) -> io::Result<()> {
        self.total = self.total.plus(value)?;
        if let Some((&first, rest)) = key.split_first() {
            let remove = {
                let child = self.children.entry(first).or_default();
                child.change(rest, value)?;
                child.total.is_zero() && child.children.is_empty()
            };
            if remove {
                self.children.remove(&first);
            }
        }
        Ok(())
    }
    pub fn prefix(&self, prefix: &[u8]) -> Value {
        let mut node = self;
        for byte in prefix {
            let Some(next) = node.children.get(byte) else {
                return Value::default();
            };
            node = next;
        }
        node.total
    }
}
