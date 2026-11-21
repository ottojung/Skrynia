use axum::body::Bytes;
use std::io::{self, Read};
use tokio::{
    sync::mpsc,
    time::{Duration, Instant, advance},
};

#[tokio::test(start_paused = true)]
async fn full_channel_send_is_deadline_bounded() {
    let (tx, mut rx) = mpsc::channel(1);
    tx.send(Ok(Bytes::from_static(b"first"))).await.unwrap();
    let (sender, deadline) = (tx.clone(), Instant::now() + Duration::from_secs(5));
    let task = tokio::spawn(async move {
        super::send::part(&sender, deadline, Bytes::from_static(b"second")).await
    });
    tokio::task::yield_now().await;
    assert!(!task.is_finished(), "full channel must apply backpressure");
    advance(Duration::from_secs(6)).await;
    assert_eq!(
        task.await.unwrap().unwrap_err().kind(),
        io::ErrorKind::TimedOut
    );
    assert_eq!(
        rx.recv().await.unwrap().unwrap(),
        Bytes::from_static(b"first")
    );
    assert!(rx.try_recv().is_err(), "no retried or duplicate frame");
}

#[test]
fn terminal_status_survives_channel_close() {
    let term = super::super::term::new();
    super::send::stop(&term, io::Error::new(io::ErrorKind::TimedOut, "stalled"));
    let (tx, rx) = mpsc::channel(1);
    drop(tx);
    let mut input = super::super::input::Input::new(rx, term);
    let mut buf = [0u8; 4];
    assert_eq!(
        input.read(&mut buf).unwrap_err().kind(),
        io::ErrorKind::TimedOut
    );
}
