use super::{QuotaIndex, Usage};
use crate::Grant;
use std::collections::HashMap;
impl QuotaIndex {
    pub fn change_metadata(
        &mut self,
        grants: &HashMap<String, Grant>,
        id: &str,
        usage: Usage,
        add: bool,
    ) {
        if !grants.contains_key(id) {
            return;
        }
        let mut next = Some(id);
        while let Some(id) = next {
            let total = self.metadata.entry(id.to_string()).or_default();
            if add {
                total.add(usage);
            } else {
                total.subtract(usage);
            }
            next = grants.get(id).and_then(|g| g.parent.as_deref());
        }
    }
}
