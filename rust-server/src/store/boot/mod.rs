mod load;
mod options;
mod verify;
use super::{Snapshot, State, locking, paths};
use std::io;
use std::sync::Mutex;

impl State {
    pub fn open() -> io::Result<Self> {
        let config = options::read()?;
        let dir = config.dir;
        let base = config.base;
        let max_object = config.max_object;
        paths::dirs(&dir)?;
        let lock = locking::acquire(&dir.join("state"))?;
        crate::recover_pending(&dir)?;
        let objects = paths::scan(&dir)?;
        let grants = load::grants(&dir)?;
        let tokens = load::tokens(&grants);
        let state = Self {
            dir,
            base,
            max_object,
            data: Mutex::new(Snapshot {
                objects,
                grants,
                tokens,
            }),
            _lock: lock,
            failed: std::sync::atomic::AtomicBool::new(false),
        };
        verify::finish(&state)?;
        Ok(state)
    }
}
