mod error;
mod finish;
pub use error::Failure;
#[cfg(test)]
mod tests;
use super::disk::{sync_dir, write_json};
use super::intent::Intent;
use std::fs;
use std::io;
use std::path::Path;

pub fn commit(
    root: &Path,
    key: &str,
    kind: &str,
    digest: Option<String>,
    change: impl FnOnce() -> io::Result<()>,
    notify: impl FnOnce() -> io::Result<()>,
) -> Result<bool, Failure> {
    let dir = root.join("state/store-transactions");
    let file = dir.join(format!("{}.json", crate::random_id()));
    let mut intent = Intent {
        key: key.into(),
        kind: kind.into(),
        sha256: digest,
        committed: false,
    };
    write_json(&file, &intent).map_err(|error| Failure::initial(&file, error))?;
    if let Err(error) = change() {
        if !intent.applied(root).map_err(Failure::Uncertain)? {
            fs::remove_file(&file).map_err(Failure::Uncertain)?;
            sync_dir(&dir).map_err(Failure::Uncertain)?;
            return Err(Failure::Clean(error));
        }
        return Err(Failure::Uncertain(error));
    }
    // Once the store directory is durable, the initial intent is sufficient
    // for crash replay while this writer lock is held.
    finish::run(&dir, &file, &mut intent, notify).map_err(Failure::Uncertain)
}
