use super::*;
use tokio::time::Duration;

#[tokio::test]
async fn full_response_queue_cannot_hold_a_worker_beyond_deadline() {
    let (tx, mut rx) = mpsc::channel(1);
    tx.try_send(Ok(Bytes::from_static(b"first"))).unwrap();
    let deadline = Instant::now() + Duration::from_millis(50);
    let worker = tokio::task::spawn_blocking(move || {
        let mut out = Output(tx, deadline);
        out.write(b"blocked")
    });
    let error = tokio::time::timeout(Duration::from_secs(2), worker)
        .await
        .expect("a blocked sender must wake without receiver activity")
        .unwrap()
        .unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::TimedOut);
    assert_eq!(rx.recv().await.unwrap().unwrap(), b"first"[..]);
    assert!(rx.recv().await.is_none());
}
