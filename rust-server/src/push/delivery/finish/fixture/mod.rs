mod create;
mod persist;
use crate::{Notification, State, Subscription};
use std::fs;
use std::path::PathBuf;
pub struct Fixture {
    pub path: PathBuf,
    pub state: State,
    pub sub: Subscription,
}
impl Fixture {
    pub fn new(generation: u64) -> Self {
        let path = std::env::temp_dir().join(format!("skrynia-test-{}", crate::random_id()));
        let sub = persist::prepare(&path, generation);
        let state = create::state(&path, &sub, generation);
        Self { path, state, sub }
    }
    pub fn read(&self) -> Notification {
        crate::read_outbox(&self.path).unwrap()[&self.sub.id].clone()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
