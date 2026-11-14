use super::completed;
use crate::Fixture;
use std::{fs, sync::atomic::Ordering};
#[tokio::test]
async fn clean_ack_failure_is_retried_without_poisoning_runner() {
    let f = Fixture::new(1);
    let before = f.read();
    let report = || {
        vec![(
            f.sub.id.clone(),
            before.clone(),
            Some(f.sub.clone()),
            Ok(()),
        )]
    };
    let path = f.path.join("state/push/outbox.json");
    fs::remove_file(&path).unwrap();
    fs::create_dir(&path).unwrap();
    assert!(completed(&f.state, report()).await);
    assert!(!f.state.failed.load(Ordering::Acquire));
    assert!(
        f.state
            .data
            .lock()
            .unwrap()
            .push
            .outbox
            .contains_key(&f.sub.id)
    );
    fs::remove_dir(&path).unwrap();
    assert!(completed(&f.state, report()).await);
    assert!(!f.state.failed.load(Ordering::Acquire));
    assert!(f.state.data.lock().unwrap().push.outbox.is_empty());
}
