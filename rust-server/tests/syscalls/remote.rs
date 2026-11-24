#[test]
fn http_roundtrip_and_exact_result_length() {
    for (body, expected) in [
        (&[0xff, 0, 0, 0][..], Some(255u32)),
        (&[0; 5][..], None),
        (&[0; 3][..], None),
    ] {
        let (url, thread) = super::provider::provider(body);
        let mut calls = libskrynia::Syscalls::new();
        calls.http_fallback(&url, &["double"]).unwrap();
        let k = super::kernel(super::ROOT, calls).unwrap();
        let result = k.invoke(b"");
        thread.join().unwrap();
        match expected {
            Some(n) => assert_eq!(result.unwrap(), n.to_le_bytes()),
            None => assert!(result.is_err()),
        }
    }
}
