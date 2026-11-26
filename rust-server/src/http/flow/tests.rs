use super::*;
use axum::body::Bytes;
use tokio::time::{Duration, timeout};

#[tokio::test]
async fn terminal_error_never_waits_for_a_full_data_queue() {
    let (tx, mut rx) = mpsc::channel(3);
    let terminal = tx.clone().try_reserve_owned().unwrap();
    tx.try_send(Ok(Bytes::from_static(b"first"))).unwrap();
    tx.try_send(Ok(Bytes::from_static(b"second"))).unwrap();
    assert!(tx.try_send(Ok(Bytes::new())).is_err());
    drop(tx);

    let pump = tokio::spawn(async { Ok(()) });
    let worker = tokio::spawn(async { Err(-5) });
    timeout(Duration::from_secs(1), supervise(pump, worker, terminal))
        .await
        .expect("terminal reporting must not wait for reader");

    assert_eq!(rx.recv().await.unwrap().unwrap(), b"first"[..]);
    assert_eq!(rx.recv().await.unwrap().unwrap(), b"second"[..]);
    assert!(rx.recv().await.unwrap().is_err());
    assert!(rx.recv().await.is_none());
}
