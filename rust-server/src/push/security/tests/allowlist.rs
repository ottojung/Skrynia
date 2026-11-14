use super::endpoint_allowed;
use crate::PushConfig;
use std::collections::HashSet;
fn config(hosts: &[&str]) -> PushConfig {
    PushConfig {
        subject: "mailto:admin@example.org".into(),
        allowed_hosts: hosts.iter().map(|x| x.to_string()).collect::<HashSet<_>>(),
        timeout_ms: 1000,
        concurrency: 1,
        max_attempts: 1,
        max_subscriptions: 10,
        max_outbox: 10,
    }
}
#[test]
fn empty_allowlist_denies_and_suffixes_match_only_subdomains() {
    let endpoint = "https://push.example.org/sub";
    assert!(!endpoint_allowed(endpoint, &config(&[])));
    assert!(endpoint_allowed(endpoint, &config(&["push.example.org"])));
    assert!(!endpoint_allowed(
        "https://notpush.example.org/sub",
        &config(&["push.example.org"])
    ));
    assert!(endpoint_allowed(
        "https://a.push.example.org/sub",
        &config(&["*.push.example.org"])
    ));
    assert!(!endpoint_allowed(
        endpoint,
        &config(&["*.push.example.org"])
    ));
    assert!(!endpoint_allowed(
        "https://evilpush.example.org/sub",
        &config(&["*.push.example.org"])
    ));
    assert!(endpoint_allowed(endpoint, &config(&["*"])));
}
