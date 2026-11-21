use super::caps::FsCaps;
use super::op::{FsOp, FsRoot};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FsAuth {
    Full,
    Empty,
    Scoped(Vec<FsCaps>),
}

impl FsAuth {
    pub fn allows(&self, root: FsRoot, op: FsOp, path: &[u8]) -> bool {
        match self {
            Self::Full => true,
            Self::Empty => false,
            Self::Scoped(caps) => caps.iter().any(|c| c.allows(root, op, path)),
        }
    }
    pub fn resolve(&self, op: FsOp, path: &[u8]) -> Option<Option<FsRoot>> {
        match self {
            Self::Full => Some(None),
            Self::Empty => None,
            Self::Scoped(caps) => {
                for root in [FsRoot::User, FsRoot::Operator] {
                    if caps.iter().any(|c| c.allows(root, op, path)) {
                        return Some(Some(root));
                    }
                }
                None
            }
        }
    }
}
