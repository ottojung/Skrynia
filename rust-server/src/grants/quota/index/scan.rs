use super::QuotaIndex;
use std::{io, path::Path};
impl QuotaIndex {
    pub fn scan_objects(&mut self, dir: &Path) -> io::Result<()> {
        crate::store::paths::visit(dir, |key, size| {
            self.scopes.change(key, None, Some(size));
            if let Some(index) = &self.delta {
                index.lock().unwrap().change(key, None, Some(size))?;
            }
            Ok(())
        })
    }
}
