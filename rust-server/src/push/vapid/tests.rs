#[test]
fn ec_keys_are_real_and_tampering_is_rejected() {
    let keys = super::generate::create().unwrap();
    super::validate::check(&keys).unwrap();
    let mut broken = keys;
    broken.private_key = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".into();
    assert!(super::validate::check(&broken).is_err());
}
