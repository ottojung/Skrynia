mod oracle;
use super::SubIndex;
use crate::{PushKeys, Subscription};
fn sub(id: &str, scope: &str, events: &[&str]) -> Subscription {
    Subscription {
        id: id.into(),
        grant_id: "0".repeat(32),
        locations: scope.into(),
        events: events.iter().map(|s| s.to_string()).collect(),
        endpoint: "https://push.example.org/test".into(),
        keys: PushKeys {
            p256dh: "BGsX0fLhLEJH-Lzm5WOkQPJ3A32BLeszoPShOUXYmMKWT-NC4v4af5uO5-tKfA-eFivOM1drMV7Oy7ZAaDe_UfU".into(),
            auth: "BwcHBwcHBwcHBwcHBwcHBw".into(),
        },
        cap_hash: "f".repeat(64),
    }
}
fn sorted(mut list: Vec<String>) -> Vec<String> {
    list.sort();
    list
}
#[test]
fn matcher_indexes_exact_prefix_and_events_without_duplicates() {
    let mut index = SubIndex::default();
    let all = sub("all", "*", &["create"]);
    let prefix = sub("prefix", "team.*", &["create", "replace"]);
    let exact = sub("exact", "team.one", &["replace"]);
    let empty = sub("empty", "", &["create"]);
    for s in [&all, &prefix, &exact, &empty] {
        index.insert(s);
    }
    assert_eq!(
        sorted(index.matching("team.one", "create")),
        ["all", "prefix"]
    );
    assert_eq!(
        sorted(index.matching("team.one", "replace")),
        ["exact", "prefix"]
    );
    assert_eq!(index.matching("unrelated", "replace"), Vec::<String>::new());
    index.remove(&prefix);
    assert_eq!(index.matching("team.one", "create"), ["all"]);
    assert_eq!(index.matching("team.one", "replace"), ["exact"]);
    index.remove(&exact);
    index.remove(&all);
    assert!(index.root.children.is_empty());
    assert!(index.matching("team.one", "replace").is_empty());
}
