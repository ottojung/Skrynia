use super::DeltaIndex;
use std::{fs, io, path::Path};
impl DeltaIndex {
    pub fn scope(&self, root: &Path, scope: &str) -> io::Result<crate::Usage> {
        if scope.is_empty() {
            return Ok(crate::Usage::default());
        }
        let Some(prefix) = scope.strip_suffix('*') else {
            let file = root.join("store").join(scope);
            return match crate::open_regular(&file) {
                Ok(f) => Ok(crate::Usage::object(f.metadata()?.len())),
                Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(crate::Usage::default()),
                Err(e) => Err(e),
            };
        };
        let mut total = self.buffer.prefix(prefix.as_bytes());
        for run in self.runs.iter().flatten() {
            total = total.plus(run.prefix(prefix.as_bytes())?)?;
        }
        total.usage()
    }
    pub fn disk_bytes(&self) -> io::Result<u64> {
        self.runs.iter().flatten().try_fold(0u64, |sum, run| {
            sum.checked_add(fs::metadata(&run.path)?.len())
                .ok_or_else(|| io::Error::other("index size overflow"))
        })
    }
}
