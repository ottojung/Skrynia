use crate::{Pending, Snapshot};
use std::io;
pub fn notifications(
    data: &Snapshot,
    ids: Vec<String>,
    max: usize,
) -> io::Result<Vec<(String, Pending)>> {
    let new = ids
        .iter()
        .filter(|id| !data.push.outbox.contains_key(*id))
        .count();
    if data.push.outbox.len().saturating_add(new) > max {
        return Err(io::Error::other("push outbox full"));
    }
    ids.into_iter()
        .map(|id| {
            let old = data.push.outbox.get(&id);
            let generation = old.map_or(Ok(1), |entry| {
                entry
                    .generation
                    .checked_add(1)
                    .ok_or_else(|| io::Error::other("generation overflow"))
            })?;
            let (attempts, next_at) =
                old.map_or((0, 0), |entry| data.push.effective_retry(&id, entry));
            Ok((
                id,
                Pending {
                    generation,
                    attempts,
                    next_at,
                },
            ))
        })
        .collect()
}
