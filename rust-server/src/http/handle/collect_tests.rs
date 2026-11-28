use super::*;
use axum::body::Body;
use futures_util::stream;
use std::io;

#[tokio::test(start_paused = true)]
async fn request_total_bounds_chunked_preflight() {
    let t = term::new();
    let v = Arc::new(AtomicU8::new(0));
    let parts = stream::unfold(0u8, |n| async move {
        if n >= 3 {
            return None;
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
        Some((Ok::<Bytes, io::Error>(Bytes::from_static(b"x")), n + 1))
    });
    let body = Body::from_stream(parts);
    let task = tokio::spawn(async move {
        buffered(
            body,
            1024,
            Duration::from_secs(5),
            Duration::from_secs(1),
            &t,
            &v,
        )
        .await
    });
    for _ in 0..4 {
        tokio::time::advance(Duration::from_secs(1)).await;
        tokio::task::yield_now().await;
    }
    assert_eq!(task.await.unwrap(), Err(StatusCode::BAD_REQUEST));
}
