mod marker;
mod ordinary;
mod pending;
use super::commit;
use std::fs;
use std::path::PathBuf;
fn root() -> PathBuf {
    let root = std::env::temp_dir().join(format!("skrynia-commit-{}", crate::random_id()));
    for dir in ["store", "store-tmp", "state/store-transactions"] {
        fs::create_dir_all(root.join(dir)).unwrap();
    }
    root
}
mod errors;
mod initial;
