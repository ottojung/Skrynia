use super::{QuotaIndex, Usage};
use crate::matches;
use std::{fs, io, path::Path};
impl QuotaIndex {
    pub fn scope_usage(dir: &Path, scope: &str) -> io::Result<Usage> {
        let mut total = Usage::default();
        if scope.is_empty() {
            return Ok(total);
        }
        for entry in fs::read_dir(dir.join("store"))? {
            let entry = entry?;
            let key = entry
                .file_name()
                .into_string()
                .map_err(|_| io::Error::other("invalid key"))?;
            if matches(scope, &key) {
                total.add(Usage::object(entry.metadata()?.len()));
            }
        }
        Ok(total)
    }
}
#[cfg(test)]
mod tests;
