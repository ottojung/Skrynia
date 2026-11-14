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

#[test]
fn grants_with_identical_scopes_share_constant_time_counters() {
    let mut index = QuotaIndex::default();
    index.scopes.insert("team.*", Usage::default());
    index.attach_grant("g1", "team.*");
    index.attach_grant("g2", "team.*");
    assert!(std::sync::Arc::ptr_eq(
        &index.by_grant["g1"],
        &index.by_grant["g2"]
    ));
    index.scopes.change("team.one", None, Some(42));
    assert_eq!(*index.by_grant["g1"].lock().unwrap(), Usage::object(42));
    assert_eq!(*index.by_grant["g2"].lock().unwrap(), Usage::object(42));
}
