use crate::{PushConfig, PushKeys, Subscription, Vapid};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use openssl::bn::BigNumContext;
use openssl::ec::{EcGroup, EcKey, PointConversionForm};
use openssl::nid::Nid;
use std::collections::HashSet;
use std::path::PathBuf;
pub fn sample(port: u16) -> (Subscription, PushConfig, Vapid, PathBuf) {
    let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1).unwrap();
    let key = EcKey::generate(&group).unwrap();
    let mut context = BigNumContext::new().unwrap();
    let public = key
        .public_key()
        .to_bytes(&group, PointConversionForm::UNCOMPRESSED, &mut context)
        .unwrap();
    let root = std::env::temp_dir().join(format!("skrynia-https-{}", crate::random_id()));
    std::fs::create_dir_all(root.join("state/push")).unwrap();
    let vapid = crate::load_vapid(&root).unwrap();
    let sub = Subscription {
        id: "a".repeat(32),
        grant_id: "b".repeat(32),
        locations: "*".into(),
        events: vec!["create".into()],
        endpoint: format!("https://push.example.org:{port}/notification"),
        keys: PushKeys {
            p256dh: URL_SAFE_NO_PAD.encode(public),
            auth: URL_SAFE_NO_PAD.encode([7u8; 16]),
        },
        cap_hash: "c".repeat(64),
    };
    let config = PushConfig {
        subject: "mailto:test@example.org".into(),
        allowed_hosts: HashSet::from(["push.example.org".into()]),
        timeout_ms: 4000,
        concurrency: 1,
        max_attempts: 2,
        max_subscriptions: 5,
        max_outbox: 5,
    };
    (sub, config, vapid, root)
}
