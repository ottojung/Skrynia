#[test]
fn http_roundtrip_and_exact_result_length() {
    for (reply, valid) in [
        (255u32.to_le_bytes().to_vec(), true),
        (vec![0; 5], false),
        (vec![0; 3], false),
    ] {
        let discovery = super::provider::Discovery {
            status: 200,
            body: r#"["double"]"#.into(),
            reply,
        };
        let provider = super::provider::start(discovery, 2);
        let mut calls = libskrynia::Syscalls::new();
        calls.http_provider(&provider.url).unwrap();
        let k = super::kernel(super::ROOT, calls).unwrap();
        let result = k.invoke(b"");
        provider.join();
        if valid {
            assert_eq!(result.unwrap(), 255u32.to_le_bytes());
        } else {
            assert!(result.is_err());
        }
    }
}
