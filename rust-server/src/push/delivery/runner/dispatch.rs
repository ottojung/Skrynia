use futures_util::{StreamExt, stream};
use std::future::Future;
pub async fn run<I, F>(work: I, capacity: usize)
where
    I: IntoIterator<Item = F>,
    F: Future<Output = ()>,
{
    stream::iter(work)
        .buffer_unordered(capacity)
        .for_each(|_| async {})
        .await;
}
#[cfg(test)]
mod tests {
    use super::run;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    #[tokio::test]
    async fn ready_work_continues_while_an_earlier_delivery_is_slow() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let work = (0..3).map(|id| {
            let events = Arc::clone(&events);
            async move {
                tokio::time::sleep(Duration::from_millis(if id == 0 { 100 } else { 5 })).await;
                events.lock().unwrap().push(id);
            }
        });
        run(work, 2).await;
        assert_eq!(*events.lock().unwrap(), vec![1, 2, 0]);
    }
}
