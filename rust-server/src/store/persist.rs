use super::State;
use std::io;
use std::path::Path;
use std::sync::atomic::Ordering;
impl State {
    pub fn require_healthy(&self) -> crate::Result<()> {
        if self.failed.load(Ordering::Acquire) {
            return Err(crate::Fail(
                axum::http::StatusCode::SERVICE_UNAVAILABLE,
                "recovery_required",
            ));
        }
        Ok(())
    }
    pub fn persist_json(&self, path: &Path, value: &impl serde::Serialize) -> crate::Result<()> {
        self.fail_on_io(crate::write_json(path, value))
    }
    pub fn persist_remove(&self, path: &Path) -> crate::Result<()> {
        self.fail_on_io(
            std::fs::remove_file(path).and_then(|_| crate::sync_dir(path.parent().unwrap())),
        )
    }
    pub fn fail_on_io<T>(&self, result: io::Result<T>) -> crate::Result<T> {
        result.map_err(|error| {
            self.failed.store(true, Ordering::Release);
            eprintln!("Skrynia filesystem persistence recovery required: {error}");
            crate::internal()
        })
    }
}
