mod boot;
mod change;
pub(crate) mod disk;
mod journal;
mod locking;
mod paths;
mod read;
mod staging;

use crate::Grant;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct State {
    pub dir: PathBuf,
    pub base: String,
    pub max_object: usize,
    pub data: Mutex<Snapshot>,
    pub _lock: File,
    pub failed: std::sync::atomic::AtomicBool,
}
pub struct Snapshot {
    pub objects: BTreeMap<String, u64>,
    pub grants: HashMap<String, Grant>,
}
