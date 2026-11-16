use super::{DeltaIndex, Value};
use std::io;
impl DeltaIndex {
    pub fn change(&mut self, key: &str, old: Option<u64>, new: Option<u64>) -> io::Result<()> {
        let delta = Value::change(old, new);
        if delta.is_zero() {
            return Ok(());
        }
        self.buffer.change(key.as_bytes(), delta)?;
        let next = self
            .pending
            .get(key)
            .copied()
            .unwrap_or_default()
            .plus(delta)?;
        if next.is_zero() {
            self.pending.remove(key);
        } else {
            self.pending.insert(key.into(), next);
        }
        self.changes = self
            .changes
            .checked_add(1)
            .ok_or_else(|| io::Error::other("index count overflow"))?;
        if self.pending.len() > self.threshold() {
            self.flush()?;
        }
        Ok(())
    }
}
