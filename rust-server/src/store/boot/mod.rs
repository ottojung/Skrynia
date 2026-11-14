mod cleanup;
mod load;
mod options;
mod outbox;
mod verify;
use super::{Snapshot, State, locking, paths};
use std::io;
use std::sync::Mutex;

impl State {
    pub fn open() -> io::Result<Self> {
        crate::validate_build_info()?;
        let config = options::read()?;
        let dir = config.dir;
        let max_object = config.max_object;
        let max_key_length = config.max_key_length;
        let max_buffered = config.max_buffered;
        let push_config = crate::PushConfig::load(&config.public)?;
        paths::dirs(&dir)?;
        let lock = locking::acquire(&dir.join("state"))?;
        outbox::initialize(&dir)?;
        cleanup::abandoned(&dir)?;
        crate::recover_pending(&dir)?;
        paths::scan(&dir)?;
        let grants = load::grants(&dir)?;
        let tokens = load::tokens(&grants);
        let push = crate::load_push(&dir)?;
        let vapid = crate::load_vapid(&dir)?;
        let state = Self {
            dir,
            base: config.base,
            max_object,
            max_key_length,
            max_buffered,
            upload: std::sync::Arc::new(tokio::sync::Semaphore::new(max_buffered)),
            push_config,
            vapid,
            data: Mutex::new(Snapshot {
                object_version: 0,
                quota: crate::QuotaIndex::default(),
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
