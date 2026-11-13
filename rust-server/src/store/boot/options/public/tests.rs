use super::base;
#[test]
fn public_url_paths_are_normalized() {
    let cases = [
        ("http://example.org", ""),
        ("HTTP://Example.Org/a//b///", "/a//b"),
        ("https://example.org/a/../b", "/b"),
        ("https://example.org/a/%2e%2e/b", "/b"),
        ("https://example.org/a/%2E/b", "/a/b"),
        ("https://example.org/a/%2f/b", "/a/%2f/b"),
        ("http://example.org/a?", "/a"),
        ("http://example.org/a#", "/a"),
    ];
    for (input, expected) in cases {
        assert_eq!(base(input).unwrap(), expected, "{input}");
    }
}
#[test]
fn invalid_and_relative_public_urls_are_rejected() {
    for input in [
        "",
        "/relative",
        "example.org/a",
        "file:///etc/passwd",
        "http://example.org/a?token=secret",
        "https://example.org/#fragment",
        "http://",
        "https://example.org:bogus/",
    ] {
        assert!(base(input).is_err(), "unexpectedly accepted {input}");
    }
}
