use super::{Fs, RootKind};

impl Fs {
    fn target(
        &self,
        root: Option<RootKind>,
        path: &[u8],
    ) -> Result<(RootKind, std::path::PathBuf), &'static str> {
        let (kind, suffix) = crate::fs::root::select::split(path)?;
        if let Some(want) = root
            && want != kind
        {
            return Err("root");
        }
        Ok((kind, crate::fs::path::clean(suffix)?))
    }
    pub fn mkdir(
        &self,
        root: Option<RootKind>,
        p: &[u8],
        mode: crate::fs::node::MkdirMode,
    ) -> Result<(), &'static str> {
        let (kind, rel) = self.target(root, p)?;
        crate::fs::node::mkdir(self.root(kind), &rel, mode)
    }
    pub fn exists(&self, root: Option<RootKind>, p: &[u8]) -> Result<bool, &'static str> {
        let (kind, rel) = self.target(root, p)?;
        crate::fs::node::exists(self.root(kind), &rel)
    }
    pub fn delete(&self, root: Option<RootKind>, p: &[u8]) -> Result<(), &'static str> {
        let (kind, rel) = self.target(root, p)?;
        crate::fs::node::delete(self.root(kind), &rel)
    }
    pub fn rename(
        &self,
        root: Option<RootKind>,
        old: &[u8],
        new: &[u8],
    ) -> Result<(), &'static str> {
        let (kind, a) = self.target(root, old)?;
        let (other, b) = self.target(root, new)?;
        if other != kind {
            return Err("root");
        }
        crate::fs::node::rename(self.root(kind), &a, &b)
    }
    pub fn is_dir(&self, root: Option<RootKind>, p: &[u8]) -> Result<bool, &'static str> {
        let (kind, rel) = self.target(root, p)?;
        crate::fs::node::is_dir(self.root(kind), &rel)
    }
}
