use super::valid;
use crate::PushKeys;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
#[test]
fn push_keys_require_a_valid_curve_point_and_auth_secret() {
    let generator = hex::decode(concat!(
        "046b17d1f2e12c4247f8bce6e563a440f277037d812deb33a0f4a13945d898c296",
        "4fe342e2fe1a7f9b8ee7eb4a7c0f9e162bce33576b315ececbb6406837bf51f5"
    ))
    .unwrap();
    let mut keys = PushKeys {
        p256dh: URL_SAFE_NO_PAD.encode(generator),
        auth: URL_SAFE_NO_PAD.encode([7u8; 16]),
    };
    assert!(valid(&keys));
    keys.p256dh = URL_SAFE_NO_PAD.encode([4u8; 65]);
    assert!(!valid(&keys));
    keys.p256dh = URL_SAFE_NO_PAD.encode([7u8; 65]);
    assert!(!valid(&keys));
    keys.auth = URL_SAFE_NO_PAD.encode([7u8; 15]);
    assert!(!valid(&keys));
}
