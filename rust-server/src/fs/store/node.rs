use super::{Fs, RootKind};

impl Fs {
    pub fn mkdir(
        &self,
        root: Option<RootKind>,
        p: &[u8],
        mode: crate::fs::node::MkdirMode,
    ) -> Result<(), &'static str> {
        let rel = crate::fs::path::clean(p)?;
        let kind = root.unwrap_or(RootKind::User);
        crate::fs::node::mkdir(self.root(kind), &rel, mode)
    }
    pub fn exists(&self, root: Option<RootKind>, p: &[u8]) -> Result<bool, &'static str> {
        let rel = crate::fs::path::clean(p)?;
        match root {
            Some(kind) => crate::fs::node::exists(self.root(kind), &rel),
            None => crate::fs::node::exists(self.root(RootKind::Operator), &rel),
        }
    }
    pub fn delete(&self, root: Option<RootKind>, p: &[u8]) -> Result<(), &'static str> {
        let rel = crate::fs::path::clean(p)?;
        let kind = root.unwrap_or(RootKind::User);
        crate::fs::node::delete(self.root(kind), &rel)
    }
    pub fn rename(
        &self,
        root: Option<RootKind>,
        old: &[u8],
        new: &[u8],
    ) -> Result<(), &'static str> {
        let a = crate::fs::path::clean(old)?;
        let b = crate::fs::path::clean(new)?;
        let kind = root.unwrap_or(RootKind::User);
        crate::fs::node::rename(self.root(kind), &a, &b)
    }
    pub fn is_dir(&self, root: Option<RootKind>, p: &[u8]) -> Result<bool, &'static str> {
        let rel = crate::fs::path::clean(p)?;
        match root {
            Some(kind) => crate::fs::node::is_dir(self.root(kind), &rel),
            None => crate::fs::node::is_dir(self.root(RootKind::Operator), &rel),
        }
    }
}
