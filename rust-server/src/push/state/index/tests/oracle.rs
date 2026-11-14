use super::{SubIndex, sub};
use crate::{Subscription, matches};
fn validate(index: &SubIndex, active: &[Subscription]) {
    for key in ["a", "ab", "abc", "xyz", "team.one", "team.two"] {
        for event in ["create", "replace", "delete"] {
            let mut expected = active
                .iter()
                .filter(|sub| matches(&sub.locations, key) && sub.events.iter().any(|e| e == event))
                .map(|sub| sub.id.clone())
                .collect::<Vec<_>>();
            let mut actual = index.matching(key, event);
            expected.sort();
            actual.sort();
            assert_eq!(actual, expected, "{key}/{event}");
        }
    }
}
#[test]
fn indexed_matches_agree_with_full_scan_after_mutations() {
    let mut index = SubIndex::default();
    let scopes = ["*", "", "a*", "ab", "abc", "team.*", "team.one", "xyz"];
    let mut active = Vec::<Subscription>::new();
    for (i, scope) in scopes.iter().enumerate() {
        let events = if i % 2 == 0 {
            &["create", "delete"][..]
        } else {
            &["replace"][..]
        };
        let subscription = sub(&format!("sub-{i}"), scope, events);
        index.insert(&subscription);
        active.push(subscription);
        validate(&index, &active);
    }
    while let Some(subscription) = active.pop() {
        index.remove(&subscription);
        validate(&index, &active);
    }
}
