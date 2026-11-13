mod index;
mod metadata;
mod objects;
mod usage;
use crate::{Fail, Result, Snapshot};
use axum::http::StatusCode;
pub use index::{QuotaIndex, Usage};
impl Snapshot {
    pub fn check_quota(&self, id: &str, key: &str, old: Option<u64>, size: u64) -> Result<()> {
        let mut next = Some(id);
        while let Some(id) = next {
            let grant = &self.grants[id];
            if grant.max_bytes.is_none() && grant.max_objects.is_none() {
                next = grant.parent.as_deref();
                continue;
            }
            let (bytes, objects) = self.usage(id, Some((key, old, size)));
            if grant.max_bytes.is_some_and(|max| bytes > max)
                || grant.max_objects.is_some_and(|max| objects > max)
            {
                return Err(Fail(
                    StatusCode::INSUFFICIENT_STORAGE,
                    "grant_quota_exceeded",
                ));
            }
            next = grant.parent.as_deref();
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests;
