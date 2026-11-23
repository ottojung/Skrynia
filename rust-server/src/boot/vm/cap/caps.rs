use super::op::{FsOp, FsRoot};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FsCaps {
    pub root: FsRoot,
    pub prefix: Vec<u8>,
    pub ops: u32,
}

impl FsCaps {
    pub fn allows(&self, root: FsRoot, op: FsOp, path: &[u8]) -> bool {
        self.root == root && self.ops & (1 << op as u32) != 0 && path_allowed(&self.prefix, path)
    }
}

pub fn path_allowed(prefix: &[u8], path: &[u8]) -> bool {
    if prefix.is_empty() {
        return true;
    }
    if path.len() < prefix.len() || &path[..prefix.len()] != prefix {
        return false;
    }
    path.len() == prefix.len() || path[prefix.len()] == b'/'
}
