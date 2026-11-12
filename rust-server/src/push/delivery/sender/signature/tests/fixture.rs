use crate::{PushConfig, PushKeys, Subscription};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use openssl::bn::BigNumContext;
use openssl::ec::{EcGroup, EcKey, PointConversionForm};
use openssl::nid::Nid;
use std::collections::HashSet;
pub fn sample() -> (Subscription, PushConfig) {
    let curve = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1).unwrap();
    let key = EcKey::generate(&curve).unwrap();
    let mut ctx = BigNumContext::new().unwrap();
    let public = key
        .public_key()
        .to_bytes(&curve, PointConversionForm::UNCOMPRESSED, &mut ctx)
        .unwrap();
    let subscriber = Subscription {
        id: "a".repeat(32),
        grant_id: "b".repeat(32),
        locations: "*".into(),
        events: vec!["create".into()],
        endpoint: "https://push.example.org/notification".into(),
        keys: PushKeys {
            p256dh: URL_SAFE_NO_PAD.encode(public),
            auth: URL_SAFE_NO_PAD.encode([7u8; 16]),
        },
        cap_hash: "c".repeat(64),
    };
    let config = PushConfig {
        subject: "mailto:test@example.org".into(),
        allowed_hosts: HashSet::new(),
        timeout_ms: 1000,
        concurrency: 1,
        max_attempts: 2,
        max_subscriptions: 5,
        max_outbox: 5,
        manual: true,
    };
    (subscriber, config)
}
