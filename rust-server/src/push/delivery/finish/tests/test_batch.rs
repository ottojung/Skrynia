use super::Fixture;
use crate::{Pending, outbox_write_count, push_finish_batch};
#[test]
fn fifty_acknowledgments_in_one_tick_have_one_durable_outbox_write() {
    let fixture = Fixture::new(1);
    let mut results = Vec::new();
    {
        let mut data = fixture.state.data.lock().unwrap();
        for index in 1..50 {
            let id = format!("{index:032x}");
            let mut sub = fixture.sub.clone();
            sub.id = id.clone();
            data.push.subscriptions.insert(id.clone(), sub);
            data.push.outbox.insert(
                id.clone(),
                Pending {
                    generation: 1,
                    attempts: 0,
                    next_at: 0,
                },
            );
        }
        crate::persist_outbox(&fixture.path, &data.push.outbox).unwrap();
        for (id, pending) in &data.push.outbox {
            let sub = data.push.subscriptions[id].clone();
            results.push((id.clone(), pending.clone(), sub));
        }
    }
    let before = outbox_write_count();
    let reports = results
        .into_iter()
        .map(|(id, notice, sub)| (id, notice, Some(sub), Ok(())))
        .collect();
    push_finish_batch(&fixture.state, reports).unwrap();
    assert_eq!(
        outbox_write_count() - before,
        1,
        "one tick caused repeated fsyncs"
    );
    assert!(crate::read_outbox(&fixture.path).unwrap().is_empty());
}
