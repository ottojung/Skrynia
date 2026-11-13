use futures_util::{FutureExt, StreamExt, stream};
use std::{future::Future, panic::AssertUnwindSafe};
pub async fn run<I, F, T, W, C>(work: I, capacity: usize, mut flush: W) -> bool
where
    I: IntoIterator<Item = F>,
    F: Future<Output = T>,
    W: FnMut(Vec<T>) -> C,
    C: Future<Output = bool>,
{
    let work = work
        .into_iter()
        .map(|future| AssertUnwindSafe(future).catch_unwind());
    let mut pending = stream::iter(work).buffer_unordered(capacity);
    let mut timer = tokio::time::interval(std::time::Duration::from_secs(1));
    timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    timer.tick().await;
    let mut completed = Vec::new();
    let mut healthy = true;
    loop {
        tokio::select! {
            next = pending.next() => match next {
                Some(Ok(item)) => completed.push(item),
                Some(Err(_)) => healthy = false,
                None => break,
            },
            _ = timer.tick(), if !completed.is_empty() => {
                if !flush(std::mem::take(&mut completed)).await { return false; }
            }
        }
    }
    if !completed.is_empty() && !flush(completed).await {
        return false;
    }
    healthy
}
#[cfg(test)]
mod tests;
