mod ctl;
mod dir;
mod disk;
mod item;
mod node;
mod ram;
pub use item::Item;
use std::sync::Arc;

pub struct Fs {
    pub root: Arc<crate::fs::root::Root>,
    pub ram: Arc<crate::ram::Ram>,
}

impl Fs {
    pub fn new(root: Arc<crate::fs::root::Root>, ram: Arc<crate::ram::Ram>) -> Self {
        Self { root, ram }
    }
}
