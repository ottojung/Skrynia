use super::DeltaIndex;
use crate::{Usage, matches};
use std::{collections::HashMap, fs};
#[test]
fn signed_runs_match_full_scan_through_merges() {
    let root = std::env::temp_dir().join(format!("skrynia-delta-{}", crate::random_id()));
    fs::create_dir_all(root.join("state")).unwrap();
    fs::create_dir_all(root.join("store")).unwrap();
    let mut index = DeltaIndex::new(&root).unwrap();
    index.limit = Some(3);
    let mut objects = HashMap::<String, u64>::new();
    let keys = ["a", "aa", "ab", "abc", "b", "bb", "xyz", "z"];
    let scopes = ["", "*", "a*", "ab*", "x*", "b*", "z*", "q*", "a"];
    for step in 0..100u64 {
        let key = keys[(step as usize * 3 + (step / 7) as usize) % keys.len()];
        let old = objects.remove(key);
        let new = (step % 5 != 0).then_some(step * 17);
        if let Some(size) = new {
            objects.insert(key.into(), size);
            fs::write(root.join("store").join(key), vec![1; size as usize]).unwrap();
        } else {
            let _ = fs::remove_file(root.join("store").join(key));
        }
        index.change(key, old, new).unwrap();
        for scope in scopes {
            let mut expected = Usage::default();
            for (name, &size) in &objects {
                if matches(scope, name) {
                    expected.add(Usage::object(size));
                }
            }
            assert_eq!(
                index.scope(&root, scope).unwrap(),
                expected,
                "{step}: {scope}"
            );
        }
    }
    assert!(index.runs.iter().any(Option::is_some));
    fs::remove_dir_all(root).unwrap();
}
