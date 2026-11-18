mod ctl;
mod dir;
mod disk;
mod item;
mod node;
mod ram;
pub use item::Item;
use std::sync::{Arc, Mutex};

pub struct Fs {
    pub root: crate::fs::root::Root,
    pub ram: Arc<crate::ram::Ram>,
    pub items: Mutex<crate::fs::table::Table<Item>>,
}

impl Fs {
    pub fn new(root: crate::fs::root::Root, ram: Arc<crate::ram::Ram>) -> Self {
        Self {
            root,
            ram,
            items: Mutex::new(crate::fs::table::Table::new()),
        }
    }
}
