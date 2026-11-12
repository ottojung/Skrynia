mod cleanup;
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
        let max_key_length = config.max_key_length;
        let max_buffered = config.max_buffered;
        let push_config = crate::PushConfig::load(&config.public)?;
        paths::dirs(&dir)?;
        let lock = locking::acquire(&dir.join("state"))?;
        cleanup::abandoned(&dir)?;
        crate::recover_pending(&dir)?;
        let objects = paths::scan(&dir)?;
        let grants = load::grants(&dir)?;
        let tokens = load::tokens(&grants);
        let mut push = crate::load_push(&dir)?;
        crate::prune_orphan_notifications(&dir, &mut push)?;
        let vapid = crate::load_vapid(&dir)?;
        let state = Self {
            dir,
            base,
            max_object,
            max_key_length,
            max_buffered,
            upload: std::sync::Arc::new(tokio::sync::Semaphore::new(max_buffered)),
            push_config,
            vapid,
            data: Mutex::new(Snapshot {
                objects,
                grants,
                tokens,
                push,
            }),
            _lock: lock,
            failed: std::sync::atomic::AtomicBool::new(false),
        };
        verify::finish(&state)?;
        Ok(state)
    }
}
