use super::{create, fixture};
use crate::{load_vapid, random_id};
use std::fs;
#[test]
fn web_push_crypto_rejects_padded_base64_keys() {
    let root = std::env::temp_dir().join(format!("skrynia-padding-{}", random_id()));
    fs::create_dir_all(root.join("state/push")).unwrap();
    let vapid = load_vapid(&root).unwrap();
    let (mut sub, config, _) = fixture::sample();
    assert!(create(&sub, &config, &vapid).is_ok());
    sub.keys.p256dh.push('=');
    assert!(create(&sub, &config, &vapid).is_err());
    sub.keys.p256dh.pop();
    sub.keys.auth.push('=');
    assert!(create(&sub, &config, &vapid).is_err());
    fs::remove_dir_all(root).unwrap();
}
