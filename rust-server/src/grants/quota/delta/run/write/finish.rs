use super::Run;
use super::Writer;
use std::io::{self, Write};
impl Writer {
    pub fn finish(mut self) -> io::Result<Run> {
        self.file.flush()?;
        self.file.get_ref().sync_all()?;
        crate::sync_dir(self.path.parent().unwrap())?;
        Ok(Run {
            path: self.path,
            len: self.count,
        })
    }
}
