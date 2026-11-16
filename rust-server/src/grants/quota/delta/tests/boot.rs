use crate::{PushState, QuotaIndex};
use std::{collections::HashMap, fs};
#[test]
fn boot_rebuilds_from_authoritative_objects_and_ignores_old_runs() {
    let root = std::env::temp_dir().join(format!("skrynia-index-{}", crate::random_id()));
    fs::create_dir_all(root.join("store")).unwrap();
    fs::create_dir_all(root.join("state")).unwrap();
    for n in 0..160u64 {
        let name = format!("{}{:03}", if n % 2 == 0 { "a" } else { "b" }, n);
        fs::write(root.join("store").join(name), vec![0; (n % 7) as usize]).unwrap();
    }
    let grants = HashMap::new();
    let push = PushState::default();
    let index = QuotaIndex::from_disk(&root, &grants, &push).unwrap();
    let delta = index.delta.as_ref().unwrap().lock().unwrap();
    assert!(delta.disk_bytes().unwrap() > 0);
    let a = delta.scope(&root, "a*").unwrap();
    assert_eq!(a.objects, 80);
    drop(delta);
    drop(index);
    fs::remove_file(root.join("store/a000")).unwrap();
    fs::write(root.join("store/a999"), vec![0; 100]).unwrap();
    let restarted = QuotaIndex::from_disk(&root, &grants, &push).unwrap();
    let delta = restarted.delta.as_ref().unwrap().lock().unwrap();
    let corrected = delta.scope(&root, "a*").unwrap();
    assert_eq!(corrected.objects, 80);
    assert_eq!(corrected.bytes, a.bytes + 100);
    fs::remove_dir_all(root).unwrap();
}
