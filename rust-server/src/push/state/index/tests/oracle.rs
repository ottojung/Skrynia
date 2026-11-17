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
#[test]
fn matcher_agrees_with_exact_oracle() {
    let mut index = SubIndex::default();
    let subs = vec![
        sub("a", "team.one", &["create", "replace"]),
        sub("b", "team.one", &["replace"]),
        sub("c", "team.two", &["delete"]),
    ];
    for s in &subs {
        index.insert(s);
    }
    for key in ["team.one", "team.two", "team", "other"] {
        for event in ["create", "replace", "delete"] {
            let mut expected: Vec<_> = subs
                .iter()
                .filter(|sub| sub.locations == key && sub.events.iter().any(|e| e == event))
                .map(|sub| sub.id.clone())
                .collect();
            expected.sort();
            let mut actual = index.matching(key, event);
            actual.sort();
            assert_eq!(actual, expected, "{key} {event}");
        }
    }
}
