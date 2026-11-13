use crate::{QuotaIndex, Usage};
use std::fs;
#[test]
fn initial_scope_scans_only_names_and_sizes() {
    let root = std::env::temp_dir().join(format!("skrynia-scope-{}", crate::random_id()));
    let store = root.join("store");
    fs::create_dir_all(&store).unwrap();
    for (key, size) in [("a", 1), ("aa", 2), ("a.b", 3), ("b", 4)] {
        fs::write(store.join(key), vec![0u8; size]).unwrap();
    }
    assert_eq!(
        QuotaIndex::scope_usage(&root, "a*").unwrap(),
        Usage {
            bytes: 6 + 3 * 512,
            objects: 3
        }
    );
    assert_eq!(
        QuotaIndex::scope_usage(&root, "*").unwrap(),
        Usage {
            bytes: 10 + 4 * 512,
            objects: 4
        }
    );
    assert_eq!(
        QuotaIndex::scope_usage(&root, "aa").unwrap(),
        Usage {
            bytes: 2 + 512,
            objects: 1
        }
    );
    assert_eq!(
        QuotaIndex::scope_usage(&root, "").unwrap(),
        Usage::default()
    );
    fs::remove_dir_all(root).unwrap();
}
