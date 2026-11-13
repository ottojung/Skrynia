use super::Fixture;
use std::sync::atomic::Ordering;

#[test]
fn push_io_failure_blocks_later_delivery_completions() {
    let fixture = Fixture::new(1);
    let pending = fixture.read();
    let file = fixture.path.join("state/push/outbox.json");
    std::fs::remove_file(&file).unwrap();
    std::fs::create_dir(&file).unwrap();
    assert!(
        crate::push_finish_batch(
            &fixture.state,
            vec![(
                fixture.sub.id.clone(),
                pending.clone(),
                Some(fixture.sub.clone()),
                Ok(())
            )]
        )
        .is_err()
    );
    assert!(fixture.state.failed.load(Ordering::Acquire));
    assert!(
        fixture
            .state
            .data
            .lock()
            .unwrap()
            .push
            .outbox
            .contains_key(&fixture.sub.id)
    );
    assert!(
        crate::push_finish_batch(
            &fixture.state,
            vec![(
                fixture.sub.id.clone(),
                pending.clone(),
                Some(fixture.sub.clone()),
                Ok(())
            )]
        )
        .is_err()
    );
}
