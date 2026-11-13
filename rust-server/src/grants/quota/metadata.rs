use crate::{Fail, Result, Snapshot};
use axum::http::StatusCode;

impl Snapshot {
    pub fn check_metadata(&self, id: &str, extra: u64) -> Result<()> {
        self.check_capacity(id, extra, 0)
    }
    pub fn check_capacity(&self, id: &str, extra: u64, count: u64) -> Result<()> {
        let mut current = Some(id);
        while let Some(id) = current {
            let grant = &self.grants[id];
            if grant.max_bytes.is_none() && grant.max_objects.is_none() {
                current = grant.parent.as_deref();
                continue;
            }
            let (bytes, objects) = self.usage(id, None);
            if grant
                .max_bytes
                .is_some_and(|max| bytes.saturating_add(extra) > max)
                || grant
                    .max_objects
                    .is_some_and(|max| objects.saturating_add(count) > max)
            {
                return Err(Fail(
                    StatusCode::INSUFFICIENT_STORAGE,
                    "grant_quota_exceeded",
                ));
            }
            current = grant.parent.as_deref();
        }
        Ok(())
    }
}
