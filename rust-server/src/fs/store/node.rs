use super::Fs;

impl Fs {
    pub fn mkdir(&self, p: &[u8], mode: crate::fs::node::MkdirMode) -> Result<(), &'static str> {
        let rel = crate::fs::path::clean(p)?;
        crate::fs::node::mkdir(&self.root, &rel, mode)
    }
    pub fn exists(&self, p: &[u8]) -> Result<bool, &'static str> {
        let rel = crate::fs::path::clean(p)?;
        crate::fs::node::exists(&self.root, &rel)
    }
    pub fn delete(&self, p: &[u8]) -> Result<(), &'static str> {
        let rel = crate::fs::path::clean(p)?;
        crate::fs::node::delete(&self.root, &rel)
    }
    pub fn rename(&self, old: &[u8], new: &[u8]) -> Result<(), &'static str> {
        let a = crate::fs::path::clean(old)?;
        let b = crate::fs::path::clean(new)?;
        crate::fs::node::rename(&self.root, &a, &b)
    }
    pub fn is_dir(&self, p: &[u8]) -> Result<bool, &'static str> {
        let rel = crate::fs::path::clean(p)?;
        crate::fs::node::is_dir(&self.root, &rel)
    }
}
