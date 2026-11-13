use futures_util::{FutureExt, StreamExt, stream};
use std::{future::Future, panic::AssertUnwindSafe};
pub async fn run<I, F, T>(work: I, capacity: usize) -> (Vec<T>, bool)
where
    I: IntoIterator<Item = F>,
    F: Future<Output = T>,
{
    let work = work
        .into_iter()
        .map(|future| AssertUnwindSafe(future).catch_unwind());
    let mut pending = stream::iter(work).buffer_unordered(capacity);
    let mut completed = Vec::new();
    let mut healthy = true;
    while let Some(result) = pending.next().await {
        match result {
            Ok(item) => completed.push(item),
            Err(_) => healthy = false,
        }
    }
    (completed, healthy)
}
#[cfg(test)]
mod tests;
