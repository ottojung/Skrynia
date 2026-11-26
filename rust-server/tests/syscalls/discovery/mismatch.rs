#[test]
fn rejects_conflicting_version_metadata() {
    let discovery = super::super::provider::Discovery {
        status: 200,
        body: super::super::provider::DOUBLE.into(),
        version: Some(
            r#"{"version":"1","commit":"development","syscalls":[{"name":"double","params":[],"results":[]}]}"#
                .into(),
        ),
        ..Default::default()
    };
    let provider = super::super::provider::start(discovery, 2);
    assert!(libskrynia::Syscalls::http(&provider.url).is_err());
    provider.join();
}

#[test]
fn rejects_non_200_and_oversized_body() {
    for (status, body) in [
        (302u16, super::super::provider::DOUBLE.to_string()),
        (200, "x".repeat(9000)),
    ] {
        let discovery = super::super::provider::Discovery {
            status,
            body,
            ..Default::default()
        };
        let provider = super::super::provider::start(discovery, 1);
        assert!(libskrynia::Syscalls::http(&provider.url).is_err());
        provider.join();
    }
}
