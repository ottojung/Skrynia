use super::DeltaIndex;
use std::{
    fs,
    io::{Seek, SeekFrom, Write},
};
#[test]
fn corrupted_run_fails_before_reporting_quota() {
    let root = std::env::temp_dir().join(format!("skrynia-index-{}", crate::random_id()));
    fs::create_dir_all(root.join("state")).unwrap();
    fs::create_dir_all(root.join("store")).unwrap();
    let mut index = DeltaIndex::new(&root).unwrap();
    index.limit = Some(1);
    index.change("a", None, Some(1)).unwrap();
    index.change("ab", None, Some(2)).unwrap();
    let run = index.runs.iter().flatten().next().unwrap();
    let mut file = fs::OpenOptions::new().write(true).open(&run.path).unwrap();
    file.seek(SeekFrom::Start(260)).unwrap();
    file.write_all(&[0x7f]).unwrap();
    file.sync_all().unwrap();
    assert!(index.scope(&root, "a*").is_err());
    fs::remove_dir_all(root).unwrap();
}
