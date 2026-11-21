use skrynia_server::boot::vm::compile::attest;

#[test]
fn malformed_precompiled_output_fails_closed() {
    let payload: &[u8] = b"precompiled-payload";
    let good = attest::wrap(payload);
    assert_eq!(attest::verify(&good), Ok(payload));
    for bad in [
        &[][..],
        &b"short"[..],
        &good[..8],
        &good[..40],
        &good[..good.len() - 1],
        &[0u8; 4096][..],
    ] {
        assert!(attest::verify(bad).is_err(), "accepted malformed envelope");
    }
    for index in [0, 7, 8, 39, good.len() - 1] {
        let mut tampered = good.clone();
        tampered[index] ^= 1;
        assert!(
            attest::verify(&tampered).is_err(),
            "accepted tampered byte {index}"
        );
    }
}
