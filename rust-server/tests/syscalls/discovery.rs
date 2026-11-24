#[test]
fn rejects_malformed_names_and_lists() {
    for body in [
        r#"["bad name"]"#,
        r#"["."]"#,
        r#""double""#,
        r#"{"syscalls":["double"]}"#,
        r#"[1]"#,
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
        (302u16, r#"["double"]"#.to_string()),
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
