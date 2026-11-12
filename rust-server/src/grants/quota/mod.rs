mod metadata;
mod objects;
mod usage;
use crate::{Fail, Result, Snapshot};
use axum::http::StatusCode;

impl Snapshot {
    pub fn check_quota(&self, id: &str, key: &str, size: u64) -> Result<()> {
        let mut next = Some(id);
        while let Some(id) = next {
            let grant = &self.grants[id];
            let (bytes, objects) = self.usage(id, Some((key, size)));
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
