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

#[test]
fn static_single_byte_chunks_cover_all_values() {
    for byte in 0..=255u8 {
        assert_eq!(chunk::bytes(&[byte]).as_ref(), &[byte]);
    }
}

#[tokio::test]
async fn batching_preserves_first_chunk_order_and_terminal_error() {
    use futures_util::StreamExt;
    let (tx, rx) = mpsc::channel(DATA_SLOTS + 1);
    let terminal = tx.clone().try_reserve_owned().unwrap();
    for byte in [b'b', b'c', b'd'] {
        tx.try_send(Ok(chunk::bytes(&[byte]))).unwrap();
    }
    terminal.send(Err(io::Error::other("terminal")));
    drop(tx);
    let mut stream = body(Bytes::from_static(b"a"), rx).into_data_stream();
    assert_eq!(stream.next().await.unwrap().unwrap(), b"a"[..]);
    assert_eq!(stream.next().await.unwrap().unwrap(), b"bcd"[..]);
    assert!(stream.next().await.unwrap().is_err());
    assert!(stream.next().await.is_none());
}
