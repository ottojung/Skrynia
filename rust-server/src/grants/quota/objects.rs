use super::{QuotaIndex, Usage};
use crate::matches;
use std::{io, path::Path};
impl QuotaIndex {
    pub fn scope_usage(dir: &Path, scope: &str) -> io::Result<Usage> {
        let mut total = Usage::default();
        if !scope.is_empty() {
            crate::store::paths::visit(dir, |key, size| {
                if matches(scope, key) {
                    total.add(Usage::object(size));
                }
                Ok(())
            })?;
        }
        Ok(total)
    }
}
