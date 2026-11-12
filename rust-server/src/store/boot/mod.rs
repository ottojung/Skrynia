mod load;
use super::{Snapshot, State, journal, locking, paths};
use std::io;
use std::sync::Mutex;

impl State {
    pub fn open() -> io::Result<Self> {
        let dir: std::path::PathBuf = std::env::var("SKRYNIA_DATA_DIR")
            .unwrap_or_else(|_| "/var/lib/skrynia".into())
            .into();
        let public =
            std::env::var("SKRYNIA_URL").map_err(|_| io::Error::other("SKRYNIA_URL required"))?;
        let url: axum::http::Uri = public.parse().map_err(io::Error::other)?;
        if !["http", "https"].contains(&url.scheme_str().unwrap_or(""))
            || url.query().is_some()
            || url.authority().is_none()
            || public.contains('#')
        {
            return Err(io::Error::other("invalid SKRYNIA_URL"));
        }
        let base = url.path().trim_end_matches('/').to_string();
        let max_object = std::env::var("SKRYNIA_MAX_OBJECT_SIZE")
            .ok()
            .and_then(|x| x.parse().ok())
            .unwrap_or(10 * 1024 * 1024);
        paths::dirs(&dir)?;
        let lock = locking::acquire(&dir.join("state"))?;
        crate::recover_pending(&dir)?;
        let objects = paths::scan(&dir)?;
        let grants = load::grants(&dir)?;
        let state = Self {
            dir,
            base,
            max_object,
            data: Mutex::new(Snapshot { objects, grants }),
            _lock: lock,
            failed: std::sync::atomic::AtomicBool::new(false),
        };
        journal::recover(&state.dir)?;
        state
            .data
            .lock()
            .map_err(|_| io::Error::other("poisoned"))?
            .validate()?;
        state.ensure_root()?;
        Ok(state)
    }
}
