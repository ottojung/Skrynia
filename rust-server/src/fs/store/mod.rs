mod ctl;
mod dir;
mod disk;
mod item;
mod node;
pub use item::Item;
use std::sync::Mutex;

pub struct Fs {
    pub root: crate::fs::root::Root,
    pub items: Mutex<crate::fs::table::Table<Item>>,
}

impl Fs {
    pub fn new(root: crate::fs::root::Root) -> Self {
        Self {
            root,
            items: Mutex::new(crate::fs::table::Table::new()),
        }
    }
}
