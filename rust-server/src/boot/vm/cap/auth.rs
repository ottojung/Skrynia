use super::caps::FsCaps;
use super::op::FsOp;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FsAuth {
    Full,
    Empty,
    Scoped(Vec<FsCaps>),
}

impl FsAuth {
    pub fn allows(&self, root: crate::fs::RootKind, op: FsOp, path: &[u8]) -> bool {
        match self {
            Self::Full => true,
            Self::Empty => false,
            Self::Scoped(caps) => caps.iter().any(|c| c.allows(root, op, path)),
        }
    }
}
