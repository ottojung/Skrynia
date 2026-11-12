mod fixture;
use super::create;
use crate::{load_vapid, random_id};
#[test]
fn vapid_and_content_encryption_produce_a_signed_request() {
    let root = std::env::temp_dir().join(format!("skrynia-sign-{}", random_id()));
    std::fs::create_dir_all(root.join("state/push")).unwrap();
    let vapid = load_vapid(&root).unwrap();
    let (subscriber, config) = fixture::sample();
    let message = create(&subscriber, &config, &vapid).unwrap();
    let request = web_push::request_builder::build_request::<Vec<u8>>(message);
    assert_eq!(request.method(), "POST");
    assert_eq!(
        request.headers().get("content-encoding").unwrap(),
        "aes128gcm"
    );
    assert!(request.headers().contains_key("authorization"));
    assert!(request.body().len() > 32);
    assert!(!request.body().windows(7).any(|part| part == b"changed"));
    std::fs::remove_dir_all(root).unwrap();
}
