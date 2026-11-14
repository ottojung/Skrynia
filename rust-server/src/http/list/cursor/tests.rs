use super::entries;
use crate::ListParams as Params;
use std::fs;
fn p(locations: &str, after: Option<&str>) -> Params {
    Params {
        locations: locations.into(),
        limit: 2,
        after: after.map(str::to_string),
    }
}
#[test]
fn lexicographic_pages_are_bounded_and_correct() {
    let root = std::env::temp_dir().join(format!("skrynia-page-{}", crate::random_id()));
    let store = root.join("store");
    fs::create_dir_all(&store).unwrap();
    for key in ["a", "abc", "abc.a", "abc.b", "abd", "z"] {
        fs::write(store.join(key), b"1").unwrap();
    }
    assert_eq!(
        entries(&root, &p("abc*", None)).unwrap(),
        ["abc", "abc.a", "abc.b"]
    );
    assert_eq!(
        entries(&root, &p("abc*", Some("abc.a"))).unwrap(),
        ["abc.b"]
    );
    assert_eq!(
        entries(&root, &p("abc*", Some("z"))).unwrap(),
        Vec::<String>::new()
    );
    assert_eq!(entries(&root, &p("abc.a", None)).unwrap(), ["abc.a"]);
    assert_eq!(
        entries(&root, &p("abc.a", Some("abc.a"))).unwrap(),
        Vec::<String>::new()
    );
    assert_eq!(entries(&root, &p("", None)).unwrap(), Vec::<String>::new());
    assert_eq!(entries(&root, &p("*", Some("abd"))).unwrap(), ["z"]);
    fs::remove_dir_all(root).unwrap();
}
