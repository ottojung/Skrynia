mod finish;
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
) -> io::Result<bool> {
    let dir = root.join("state/store-transactions");
    let file = dir.join(format!("{}.json", crate::random_id()));
    let mut intent = Intent {
        key: key.into(),
        kind: kind.into(),
        sha256: digest,
        committed: false,
    };
    write_json(&file, &intent)?;
    if let Err(error) = change() {
        // An applied mutation may be recoverable, so preserve its intent.
        if !intent.applied(root)? {
            fs::remove_file(&file)?;
            sync_dir(&dir)?;
        }
        return Err(error);
    }
    // Once the store directory is durable, the initial intent is sufficient
    // for crash replay while this writer lock is held.
    finish::run(&dir, &file, &mut intent, notify)
}
