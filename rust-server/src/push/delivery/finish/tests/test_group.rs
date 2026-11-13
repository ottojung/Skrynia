use super::Fixture;
use crate::{push_finish, write_json};
use std::fs;
#[test]
fn successful_ack_removes_snapshot_only_notification_durably() {
    let f = Fixture::new(1);
    let entry = f.read();
    let base = f
        .path
        .join("state/push/outbox")
        .join(format!("{}.json", f.sub.id));
    fs::remove_file(base).unwrap();
    let index = f.path.join("state/push/outbox-generations.json");
    write_json(
        &index,
        &serde_json::json!({f.sub.id.clone():entry.generation}),
    )
    .unwrap();
    push_finish(&f.state, &entry, Some(&f.sub), Ok(())).unwrap();
    let generations: serde_json::Value = serde_json::from_slice(&fs::read(index).unwrap()).unwrap();
    assert_eq!(generations, serde_json::json!({}));
    assert!(f.state.data.lock().unwrap().push.outbox.is_empty());
    assert!(crate::load_push(&f.path).unwrap().outbox.is_empty());
}
