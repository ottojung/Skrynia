use super::{Fs, Item};

impl Fs {
    pub fn open_directory(&self, p: &[u8]) -> Result<crate::fs::Fd, &'static str> {
        let rel = crate::fs::path::clean(p)?;
        let e = crate::fs::dir::open_dir(&self.root, &rel)?;
        Ok(self.items.lock().unwrap().insert(Item::Dir(e)))
    }
    pub fn next_entry(
        &self,
        fd: crate::fs::Fd,
        dst: &mut [u8],
    ) -> Result<crate::fs::dir::Next, &'static str> {
        let mut t = self.items.lock().unwrap();
        match t.get_mut(fd) {
            Some(Item::Dir(e)) => crate::fs::dir::next_entry(e, dst),
            _ => Err("badfd"),
        }
    }
}
