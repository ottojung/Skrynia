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
    if let Err(error) = finish(&dir, &file, &mut intent) {
        eprintln!("Skrynia committed object with pending journal recovery: {error}");
        return Ok(true);
    }
    Ok(false)
}
fn finish(dir: &Path, file: &Path, intent: &mut Intent) -> io::Result<()> {
    intent.committed = true;
    write_json(file, intent)?;
    fs::remove_file(file)?;
    sync_dir(dir)
}
