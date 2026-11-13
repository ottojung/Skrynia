use futures_util::{FutureExt, StreamExt, stream};
use std::{future::Future, panic::AssertUnwindSafe};
pub async fn run<I, F>(work: I, capacity: usize) -> bool
where
    I: IntoIterator<Item = F>,
    F: Future<Output = ()>,
{
    let work = work
        .into_iter()
        .map(|future| AssertUnwindSafe(future).catch_unwind());
    let mut pending = stream::iter(work).buffer_unordered(capacity);
    let mut healthy = true;
    while let Some(result) = pending.next().await {
        if result.is_err() {
            healthy = false;
        }
    }
    healthy
}
#[cfg(test)]
mod tests;
