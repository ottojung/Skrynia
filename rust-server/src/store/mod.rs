mod boot;
mod change;
pub(crate) mod disk;
mod journal;
mod locking;
mod paths;
mod persist;
mod read;
mod staging;

use crate::{Grant, PushConfig, PushState, Vapid};
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct State {
    pub dir: PathBuf,
    pub base: String,
    pub max_object: usize,
    pub max_key_length: usize,
    pub max_buffered: usize,
    pub upload: Arc<tokio::sync::Semaphore>,
    pub push_config: PushConfig,
    pub vapid: Vapid,
    pub data: Mutex<Snapshot>,
    pub _lock: File,
    pub failed: std::sync::atomic::AtomicBool,
}
pub struct Snapshot {
    pub objects: BTreeMap<String, u64>,
    pub grants: HashMap<String, Grant>,
    pub tokens: HashMap<String, String>,
    pub push: PushState,
}
