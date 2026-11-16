use super::DeltaIndex;
use crate::Usage;
use std::fs;
#[test]
fn restart_discards_untrusted_derived_runs() {
    let root = std::env::temp_dir().join(format!("skrynia-delta-{}", crate::random_id()));
    fs::create_dir_all(root.join("state")).unwrap();
    fs::create_dir_all(root.join("store")).unwrap();
    let mut index = DeltaIndex::new(&root).unwrap();
    index.limit = Some(1);
    index.change("a", None, Some(1)).unwrap();
    index.change("aa", None, Some(2)).unwrap();
    assert!(index.disk_bytes().unwrap() > 0);
    let mut rebuilt = DeltaIndex::new(&root).unwrap();
    assert_eq!(rebuilt.scope(&root, "a*").unwrap(), Usage::default());
    rebuilt.change("a", None, Some(1)).unwrap();
    rebuilt.change("aa", None, Some(2)).unwrap();
    assert_eq!(
        rebuilt.scope(&root, "a*").unwrap(),
        Usage {
            bytes: 1027,
            objects: 2
        }
    );
    fs::remove_dir_all(root).unwrap();
}
