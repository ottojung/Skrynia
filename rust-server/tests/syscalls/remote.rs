#[test]
fn http_roundtrip_and_exact_result_length() {
    let expected = super::provider::wire(&[(b'i', 255)]);
    for (reply, valid) in [
        (expected.clone(), true),
        (vec![0; 21], false),
        (vec![0; 19], false),
    ] {
        let discovery = super::provider::Discovery {
            status: 200,
            body: super::provider::DOUBLE.into(),
            reply,
            request: Some(super::provider::wire(&[(b'i', u64::MAX)])),
            ..Default::default()
        };
        let provider = super::provider::start(discovery, 3);
        let calls = libskrynia::Syscalls::http(&provider.url).unwrap();
        let k = super::kernel(super::ROOT, calls).unwrap();
        let result = k.invoke(b"");
        provider.join();
        if valid {
            assert_eq!(result.unwrap(), vec![255, 0, 0, 0]);
        } else {
            assert!(result.is_err());
        }
    }
}
