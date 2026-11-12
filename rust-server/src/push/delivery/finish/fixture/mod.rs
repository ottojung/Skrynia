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
        serde_json::from_slice(
            &fs::read(
                self.path
                    .join("state/push/outbox")
                    .join(format!("{}.json", self.sub.id)),
            )
            .unwrap(),
        )
        .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
