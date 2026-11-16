use super::{Value, buffer::Node, run::Run};
use std::{collections::BTreeMap, path::PathBuf};
pub struct DeltaIndex {
    pub(super) dir: PathBuf,
    pub(super) pending: BTreeMap<String, Value>,
    pub(super) buffer: Node,
    pub(super) runs: Vec<Option<Run>>,
    pub(super) changes: u64,
    pub(super) limit: Option<usize>,
}
impl DeltaIndex {
    pub fn threshold(&self) -> usize {
        self.limit.unwrap_or_else(|| {
            usize::try_from(self.changes.max(1).isqrt())
                .unwrap_or(4096)
                .clamp(64, 4096)
        })
    }
}
