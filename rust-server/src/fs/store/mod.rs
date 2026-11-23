mod ctl;
mod dir;
mod disk;
pub mod item;
mod node;
mod ram;
pub use item::Item;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RootKind {
    Operator,
    User,
}

pub struct Fs {
    pub operator: Arc<crate::fs::root::Root>,
    pub user: Arc<crate::fs::root::Root>,
    pub ram: Arc<crate::ram::Ram>,
}

impl Fs {
    pub fn new(
        operator: Arc<crate::fs::root::Root>,
        user: Arc<crate::fs::root::Root>,
        ram: Arc<crate::ram::Ram>,
    ) -> Self {
        Self {
            operator,
            user,
            ram,
        }
    }
    pub fn root(&self, kind: RootKind) -> &crate::fs::root::Root {
        match kind {
            RootKind::Operator => &self.operator,
            RootKind::User => &self.user,
        }
    }
    pub fn pick_read_root(&self, _path: &[u8]) -> RootKind {
        RootKind::Operator
    }
}
