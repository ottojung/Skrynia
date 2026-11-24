#[test]
fn rejects_malformed_and_reserved_names() {
    for body in [
        r#"{"syscalls":["bad name"]}"#,
        r#"{"syscalls":["."]}"#,
        r#"{"syscalls":["version"]}"#,
        r#"{"syscalls":"double"}"#,
        r#"{"syscalls":[1]}"#,
        r#"{}"#,
        r#"not json"#,
    ] {
        let d = super::provider::Discovery {
            status: 200,
            body: body.into(),
            reply: Vec::new(),
        };
        let provider = super::provider::start(d, 1);
        let mut calls = libskrynia::Syscalls::new();
        assert!(calls.http_provider(&provider.url).is_err());
        provider.join();
    }
}

#[test]
fn rejects_non_200_and_oversized_body() {
    for (status, body) in [
        (302u16, r#"{"syscalls":["double"]}"#.to_string()),
        (200, "x".repeat(9000)),
    ] {
        let d = super::provider::Discovery {
            status,
            body,
            reply: Vec::new(),
        };
        let provider = super::provider::start(d, 1);
        let mut calls = libskrynia::Syscalls::new();
        assert!(calls.http_provider(&provider.url).is_err());
        provider.join();
    }
}
