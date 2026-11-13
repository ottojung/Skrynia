mod decrypt;
use super::{create, fixture};
use crate::{load_vapid, random_id};
#[test]
fn browser_style_receiver_decrypts_changed_and_rejects_tampering() {
    let root = std::env::temp_dir().join(format!("skrynia-browser-{}", random_id()));
    std::fs::create_dir_all(root.join("state/push")).unwrap();
    let vapid = load_vapid(&root).unwrap();
    let (subscriber, config, receiver) = fixture::sample();
    let message = create(&subscriber, &config, &vapid).unwrap();
    let request = web_push::request_builder::build_request::<Vec<u8>>(message);
    let encrypted = request.body();
    assert_eq!(decrypt::decode(&receiver, encrypted).unwrap(), b"changed");
    let mut corrupted = encrypted.to_vec();
    *corrupted.last_mut().unwrap() ^= 0x80;
    assert!(decrypt::decode(&receiver, &corrupted).is_err());
    let (_, _, wrong_receiver) = fixture::sample();
    assert!(decrypt::decode(&wrong_receiver, encrypted).is_err());
    std::fs::remove_dir_all(root).unwrap();
}
