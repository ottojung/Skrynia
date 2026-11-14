mod create;
mod persist;
use crate::{Pending, State, Subscription};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
pub struct Fixture {
    pub path: PathBuf,
    pub state: Arc<State>,
    pub sub: Subscription,
}
impl Fixture {
    pub fn new(generation: u64) -> Self {
        let path = std::env::temp_dir().join(format!("skrynia-test-{}", crate::random_id()));
        let sub = persist::prepare(&path, generation);
        let state = Arc::new(create::state(&path, &sub, generation));
        Self { path, state, sub }
    }
    pub fn read(&self) -> Pending {
        crate::read_outbox(&self.path).unwrap()[&self.sub.id].clone()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
