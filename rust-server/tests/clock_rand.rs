use skrynia_server::{clock, rand};

#[test]
fn now_is_recent() {
    let t = clock::now_millis();
    assert!(t > 1_700_000_000_000);
}

#[test]
fn random_fills_and_varies() {
    let mut a = [0u8; 32];
    let mut b = [0u8; 32];
    rand::fill_secure(&mut a).unwrap();
    rand::fill_secure(&mut b).unwrap();
    assert_ne!(a, [0u8; 32]);
    assert_ne!(a, b);
}
