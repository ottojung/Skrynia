use super::entries;
use crate::ListParams as Params;
use crate::{PushState, Snapshot};
use std::collections::{BTreeMap, HashMap};
fn state() -> Snapshot {
    let objects = ["a", "abc", "abc.a", "abc.b", "abd", "z"]
        .into_iter()
        .map(|s| (s.to_string(), 1))
        .collect::<BTreeMap<_, _>>();
    Snapshot {
        objects,
        grants: HashMap::new(),
        tokens: HashMap::new(),
        push: PushState::default(),
    }
}
fn p(locations: &str, after: Option<&str>) -> Params {
    Params {
        locations: locations.into(),
        limit: 2,
        after: after.map(str::to_string),
    }
}
#[test]
fn lexicographic_pages_are_bounded_and_correct() {
    let state = state();
    assert_eq!(entries(&state, &p("abc*", None)), ["abc", "abc.a", "abc.b"]);
    assert_eq!(entries(&state, &p("abc*", Some("abc.a"))), ["abc.b"]);
    assert_eq!(entries(&state, &p("abc*", Some("z"))), Vec::<String>::new());
    assert_eq!(entries(&state, &p("abc.a", None)), ["abc.a"]);
    assert_eq!(
        entries(&state, &p("abc.a", Some("abc.a"))),
        Vec::<String>::new()
    );
    assert_eq!(entries(&state, &p("", None)), Vec::<String>::new());
    assert_eq!(entries(&state, &p("*", Some("abd"))), ["z"]);
}
