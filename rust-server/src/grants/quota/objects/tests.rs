use super::sum;
use std::collections::BTreeMap;
#[test]
fn scopes_and_updates() {
    let objects: BTreeMap<String, u64> = [("a", 1), ("aa", 2), ("a.b", 3), ("b", 4)]
        .into_iter()
        .map(|(n, v)| (n.into(), v))
        .collect();
    assert_eq!(sum(&objects, "", None), (0, 0));
    assert_eq!(sum(&objects, "a", None), (513, 1));
    assert_eq!(sum(&objects, "a*", None), (1 + 2 + 3 + 3 * 512, 3));
    assert_eq!(sum(&objects, "*", None), (1 + 2 + 3 + 4 + 4 * 512, 4));
    assert_eq!(
        sum(&objects, "a*", Some(("ab", 9))),
        (1 + 2 + 3 + 9 + 4 * 512, 4)
    );
    assert_eq!(
        sum(&objects, "a*", Some(("aa", 9))),
        (1 + 9 + 3 + 3 * 512, 3)
    );
    assert_eq!(
        sum(&objects, "a*", Some(("b", 9))),
        (1 + 2 + 3 + 3 * 512, 3)
    );
}
