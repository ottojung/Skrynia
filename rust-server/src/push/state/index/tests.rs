mod oracle;
pub(super) use super::SubIndex;
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
fn matcher_indexes_exact_keys_and_events_without_duplicates() {
    let mut index = SubIndex::default();
    let one = sub("one", "team.one", &["create", "replace"]);
    let other = sub("other", "team.one", &["replace"]);
    let unrelated = sub("unrelated", "team.two", &["create"]);
    for s in [&one, &other, &unrelated] {
        index.insert(s);
    }
    assert_eq!(sorted(index.matching("team.one", "create")), ["one"]);
    assert_eq!(
        sorted(index.matching("team.one", "replace")),
        ["one", "other"]
    );
    assert!(index.matching("team.two", "replace").is_empty());
    assert!(index.matching("team", "create").is_empty());
    index.remove(&one);
    assert_eq!(index.matching("team.one", "create"), Vec::<String>::new());
    assert_eq!(index.matching("team.one", "replace"), ["other"]);
    index.remove(&other);
    index.remove(&unrelated);
    assert!(index.matching("team.one", "replace").is_empty());
    assert!(index.matching("team.two", "create").is_empty());
}
