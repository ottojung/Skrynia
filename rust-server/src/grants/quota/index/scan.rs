use super::QuotaIndex;
use std::{fs, io, path::Path};
impl QuotaIndex {
    pub fn scan_objects(&mut self, dir: &Path) -> io::Result<()> {
        for entry in fs::read_dir(dir.join("store"))? {
            let entry = entry?;
            let key = entry
                .file_name()
                .into_string()
                .map_err(|_| io::Error::other("invalid key"))?;
            if !crate::valid_key(&key) || !entry.file_type()?.is_file() {
                return Err(io::Error::other("invalid stored object"));
            }
            self.scopes
                .change(&key, None, Some(entry.metadata()?.len()));
        }
        Ok(())
    }
}
