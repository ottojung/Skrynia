use crate::{QuotaIndex, Usage};
#[test]
fn prefix_counters_follow_overlapping_mutations() {
    let mut index = QuotaIndex::default();
    index.scopes.insert("*", Usage::default());
    index.scopes.insert("a*", Usage::default());
    index.scopes.insert("ab*", Usage::default());
    index.scopes.insert("ab", Usage::default());
    index.scopes.change("ab", None, Some(100));
    assert_eq!(index.scopes.get("ab*"), Usage::object(100));
    assert_eq!(index.scopes.get("ab"), Usage::object(100));
    index.scopes.change("ab", Some(100), Some(3));
    index.scopes.change("ax", None, Some(7));
    index.scopes.change("ab", Some(3), None);
    assert_eq!(index.scopes.get("ab"), Usage::default());
    assert_eq!(index.scopes.get("ab*"), Usage::default());
    assert_eq!(index.scopes.get("a*"), Usage::object(7));
    assert_eq!(index.scopes.get("*"), Usage::object(7));
}
