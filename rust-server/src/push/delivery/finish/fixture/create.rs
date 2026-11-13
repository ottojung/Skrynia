use crate::{Pending, PushConfig, PushState, Snapshot, State, Subscription, Vapid};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::sync::Mutex;
pub fn state(path: &Path, sub: &Subscription, generation: u64) -> State {
    let notice = Pending {
        generation,
        attempts: 0,
        next_at: 0,
    };
    let mut push = PushState {
        subscriptions: HashMap::from([(sub.id.clone(), sub.clone())]),
        outbox: HashMap::from([(sub.id.clone(), notice)]),
        index: Default::default(),
    };
    push.reindex();
    let config = PushConfig {
        subject: "mailto:test@example.org".into(),
        allowed_hosts: HashSet::new(),
        timeout_ms: 1000,
        concurrency: 1,
        max_attempts: 3,
        max_subscriptions: 10,
        max_outbox: 10,
    };
    State {
        dir: path.to_path_buf(),
        base: String::new(),
        max_object: 100,
        max_key_length: 256,
        max_buffered: 100,
        upload: std::sync::Arc::new(tokio::sync::Semaphore::new(100)),
        data: Mutex::new(Snapshot {
            quota: crate::QuotaIndex::default(),
            grants: HashMap::new(),
            tokens: HashMap::new(),
            push,
        }),
        push_config: config,
        vapid: Vapid {
            public_key: String::new(),
            private_key: String::new(),
        },
        _lock: fs::File::create(path.join("lock")).unwrap(),
        failed: std::sync::atomic::AtomicBool::new(false),
    }
}
